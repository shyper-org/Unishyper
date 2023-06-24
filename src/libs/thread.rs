use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::sync::Arc;
use core::fmt;
use core::mem;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use core::cell::UnsafeCell;

use spin::{Mutex, Once};

use crate::arch::{ContextFrame, PAGE_SIZE, STACK_SIZE, ThreadContext};
use crate::libs::traits::ContextFrameTrait;
use crate::libs::cpu::{CoreId, cpu, get_cpu};
use crate::libs::scheduler::Scheduler;
use crate::libs::error::*;
use crate::libs::synch::spinlock::SpinlockIrqSave;
use crate::mm::address::VAddr;
use crate::mm::stack::Stack;
use crate::mm::paging::MappedRegion;
use crate::util::{round_up, irqsave};

pub type Tid = usize;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PrivilegedLevel {
    User,
    Kernel,
}

impl fmt::Display for PrivilegedLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PrivilegedLevel::User => write!(f, "USER"),
            PrivilegedLevel::Kernel => write!(f, "KERNEL"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Status {
    Runnable,
    Ready,
    Blocked,
    Exited,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Status::Runnable => write!(f, "Running"),
            Status::Ready => write!(f, "Ready"),
            Status::Blocked => write!(f, "Blocked"),
            Status::Exited => write!(f, "Exited"),
        }
    }
}

#[derive(Debug)]
struct Inner {
    uuid: usize,
    level: PrivilegedLevel,
    #[allow(unused)]
    stack: Stack,
    tls: crate::libs::tls::ThreadTls,
}

struct InnerMut {
    // Todo: these Mutexes may be removed.
    affinity_core: Option<CoreId>,
    // running_core: Mutex<CoreId>,
    status: Mutex<Status>,
    trap_stack_pointer: Mutex<usize>,
    in_trap_context: Mutex<bool>,
    ctx: UnsafeCell<ThreadContext>,
    mem_regions: Mutex<BTreeMap<VAddr, MappedRegion>>,
}

unsafe impl Send for InnerMut {}
unsafe impl Sync for InnerMut {}

struct ControlBlock {
    inner: Inner,
    inner_mut: InnerMut,
}

impl Drop for ControlBlock {
    fn drop(&mut self) {
        debug!("Drop Thread [{}]'s ControlBlock", self.inner.uuid);
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Thread(Arc<ControlBlock>);

impl Ord for Thread {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.inner.uuid.cmp(&other.0.inner.uuid)
    }
}

impl PartialOrd for Thread {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Thread {
    fn eq(&self, other: &Self) -> bool {
        self.0.inner.uuid == other.0.inner.uuid
    }
}

impl Eq for Thread {}
// impl Drop for Thread {
//     fn drop(&mut self) {
//         debug!("Drop Thread [{}]'s struct, TCB Arc stong count {}", self.tid(), Arc::strong_count(&self.0));
//     }
// }

impl fmt::Debug for Thread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Thread")
            .field("id", &self.0.inner.uuid)
            .field("stack", &self.0.inner.stack.start_address())
            .field("state", &self.status())
            .finish()
    }
}

impl Thread {
    /// Get thread tid, which is globally unique.
    pub fn tid(&self) -> Tid {
        self.0.inner.uuid
    }

    pub fn affinity_core(&self) -> Option<CoreId> {
        self.0.inner_mut.affinity_core
    }

    // pub fn set_core_id(&self, target_core_id: CoreId) {
    //     let mut lock = self.0.inner_mut.core_id.lock();
    //     *lock = target_core_id
    // }

    /// Get thread status.
    pub fn status(&self) -> Status {
        let lock = self.0.inner_mut.status.lock();
        lock.clone()
    }

    /// Get if thread is runnable.
    pub fn runnable(&self) -> bool {
        let lock = self.0.inner_mut.status.lock();
        *lock == Status::Runnable
    }

    pub fn set_exited(&mut self) {
        let mut lock = self.0.inner_mut.status.lock();
        *lock = Status::Exited;
    }

    /// Get thread privilege level.
    pub fn privilege(&self) -> PrivilegedLevel {
        self.0.inner.level
    }

    pub fn set_last_stack_pointer(&self, sp: usize) {
        let mut trap_stack_pointer = self.0.inner_mut.trap_stack_pointer.lock();
        *trap_stack_pointer = sp;
    }

    pub fn last_stack_pointer(&self) -> usize {
        let trap_stack_pointer = self.0.inner_mut.trap_stack_pointer.lock();
        *trap_stack_pointer
    }

    #[inline]
    pub unsafe fn ctx_mut_ptr(&self) -> *mut ThreadContext {
        self.0.inner_mut.ctx.get()
    }

    #[inline]
    pub fn in_trap_context(&self) -> bool {
        let in_trap_context = self.0.inner_mut.in_trap_context.lock();
        *in_trap_context
    }

    #[inline]
    pub fn set_in_yield_context(&self) {
        let mut in_trap_context = self.0.inner_mut.in_trap_context.lock();
        *in_trap_context = false;
    }

    /// Add newly allocated MappedRegion to thread's control block.
    /// The ownership of region is token over by this thread.
    /// The region is automitically dropped(unmapped) when thread is destroied.
    pub fn add_mem_region(&self, addr: VAddr, region: MappedRegion) {
        let mut addr_space = self.0.inner_mut.mem_regions.lock();
        addr_space.insert(addr, region);
    }

    /// Remove target MappedRegion from thread's control block according to addr.
    /// The ownership of region is transfered from this thread.
    /// The freed region will be automically dropped.
    pub fn free_mem_region(&self, addr: VAddr) {
        let mut addr_space = self.0.inner_mut.mem_regions.lock();
        addr_space.remove(&addr);
    }

    /// Get thread local storage region's start address.
    /// We currently store the tls pointer in tpidr_el0 as aarch64 normally does.
    /// See src/arch/tls.rs for more details.
    pub fn get_tls_ptr(&self) -> *const u8 {
        self.0.inner.tls.get_tls_start().as_ptr::<u8>()
    }
}

static THREAD_UUID_ALLOCATOR: AtomicUsize = AtomicUsize::new(100);

/// Alloc global unique id for new thread.
fn new_tid() -> Tid {
    THREAD_UUID_ALLOCATOR.fetch_add(1, Ordering::Relaxed)
}

/// Store thread IDs and its corresponding thread struct.
static THREAD_MAP: Mutex<BTreeMap<Tid, Thread>> = Mutex::new(BTreeMap::new());

/// Store thread IDs and its corresponding thread name.
/// It doesn't store all threads' information, cause not all thread have their name.
/// Generally only threads on the background may exist here.
static THREAD_NAME_MAP: Mutex<BTreeMap<Tid, String>> = Mutex::new(BTreeMap::new());

/// Store thread IDs and its corresponding waiting threads' queue.
static THREAD_WAITING_QUEUE: SpinlockIrqSave<BTreeMap<Tid, VecDeque<Thread>>> =
    SpinlockIrqSave::new(BTreeMap::new());

static THREAD_EXIT_QUEUE: Once<Mutex<VecDeque<Thread>>> = Once::new();

fn thread_exit_queue() -> &'static Mutex<VecDeque<Thread>> {
    match THREAD_EXIT_QUEUE.get() {
        None => THREAD_EXIT_QUEUE.call_once(|| Mutex::new(VecDeque::new())),
        Some(x) => x,
    }
}

/// List background threads' ids and names infornation.
pub fn list_threads() {
    let name_map = THREAD_NAME_MAP.lock();
    let thread_map = THREAD_MAP.lock();
    println!(" [ TID] STATUS\tPRI\tNAME");
    for t in thread_map.clone().into_iter() {
        let name = match name_map.get(&t.0) {
            Some(name) => name.as_str(),
            None => "system-thread",
        };
        println!(
            "-[{:4}] {}\t{}\t{:?}",
            t.0,
            t.1.status(),
            t.1.privilege(),
            name
        );
    }
}

/// This is the main thread alloc logic, which contains the following logic.
/// 1.  generate new thread id(or use the given thread id);
/// 2.  alloc mapped memory region for stack according to stack size;
/// 3.  construct thread control block, including inner and inner_mut;
/// 4.  init context frame inside the thread's stack region, including entry, sp, e.g.
/// 5.  insert thread struct into glocal THREAD_MAP;
/// 6.  return the generated Thread struct.
///
/// ## Arguments
///
/// * `id`        - Expected thread id, if None this function will call new_tid to atomicly generate a new one.
/// * `start`     - Thread's entry address, a wrapper of thread, generally it's set as `thread_start`, see _inner_spawn for details.
/// * `entry`     - Thread's first executed function, it's the true entry inside the wrapper.
/// * `arg`       - Thread's first argument.
/// * `privilege` - Thread's privilige level, if true the thread is set as KERNEL thread, which can not be killed by user.
///
/// Notes: the generated thread is at Ready state, you need to wake it up.
pub fn thread_alloc(
    id: Option<usize>,
    affinity_core: Option<CoreId>,
    start: usize,
    entry: usize,
    arg: usize,
    privilege: bool,
) -> Thread {
    // Generally it should call the new_tid function to get a newly generated id,
    // During thread_restart, the reallocated thread may use its original id.
    let id = id.unwrap_or(new_tid());

    // pub const STACK_SIZE: usize = 32_768; // PAGE_SIZE * 8
    let stack_size = round_up(STACK_SIZE, PAGE_SIZE);

    let stack_region = crate::mm::stack::alloc_stack(stack_size / PAGE_SIZE, id)
        .expect("fail to allocate user thread stack");
    let stack_start = stack_region.start_address();

    let sp = stack_start + stack_region.size_in_bytes();

    let last_stack_pointer = sp - mem::size_of::<ContextFrame>();

    debug!(
        "thread alloc sp {:#x} last_stack_pointer {:#x} size of ContextFrame {:#x}",
        sp,
        last_stack_pointer,
        mem::size_of::<ContextFrame>()
    );
    // Init thread context in stack region.
    unsafe {
        #[cfg(all(target_arch = "x86_64", feature = "mpk"))]
        let ori_pkru = crate::arch::mpk::swicth_to_kernel_pkru();

        core::ptr::write_bytes(
            last_stack_pointer.as_mut_ptr::<u8>(),
            0,
            mem::size_of::<ContextFrame>(),
        );
        let context_frame = &mut *last_stack_pointer
            .as_mut_ptr::<ContextFrame>()
            .as_mut()
            .unwrap();
        context_frame.init(id);
        context_frame.set_exception_pc(start);
        context_frame.set_gpr(0, entry);
        context_frame.set_gpr(1, arg);
        context_frame.set_stack_pointer(sp.value());
        trace!(
            "NEW context_frame: on {:#p} \n{}",
            context_frame,
            context_frame
        );
        #[cfg(all(target_arch = "x86_64", feature = "mpk"))]
        crate::arch::mpk::switch_from_kernel_pkru(ori_pkru);
    }

    // Init thread local storage region.
    let tls = crate::libs::tls::alloc_thread_local_storage_region();
    debug!("tls_region alloc at {}", tls.get_tls_start());

    let t = Thread(Arc::new(ControlBlock {
        inner: Inner {
            uuid: id,
            level: if privilege {
                PrivilegedLevel::Kernel
            } else {
                PrivilegedLevel::User
            },
            stack: stack_region,
            tls,
        },
        inner_mut: InnerMut {
            affinity_core,
            status: Mutex::new(Status::Ready),
            trap_stack_pointer: Mutex::new(last_stack_pointer.value()),
            ctx: UnsafeCell::new(ThreadContext::new()),
            in_trap_context: Mutex::new(true),
            mem_regions: Mutex::new(BTreeMap::new()),
        },
    }));
    let mut map = THREAD_MAP.lock();
    map.insert(id, t.clone());

    THREAD_WAITING_QUEUE
        .lock()
        .insert(id, VecDeque::with_capacity(1));

    debug!(
        "thread_alloc success id [{}]\n\t\t\t\t\t\tsp [{} to 0x{:016x}]",
        id, stack_start, sp
    );
    t
}

/// Find target thread by thread id.
/// Return None if thread not exist.
pub fn thread_lookup(tid: Tid) -> Option<Thread> {
    let map = THREAD_MAP.lock();
    map.get(&tid).cloned()
}

/// Destory target thread.
/// Remove it from THREAD_NAME_MAP and THREAD_MAP.
#[inline(always)]
pub fn thread_destroy(t: &Thread) {
    debug!("Destroy t{}", t.tid());
    if let Some(current_thread) = cpu().running_thread() {
        if t.tid() == current_thread.tid() {
            cpu().set_running_thread(None);
        }
    }
    let mut name_map = THREAD_NAME_MAP.lock();
    name_map.remove(&t.tid());
    let mut map = THREAD_MAP.lock();
    map.remove(&t.tid());
    THREAD_WAITING_QUEUE.lock().remove(&t.tid());
}

/// Destory target thread by thread id.
/// See thread_destroy for more details.
pub fn thread_destroy_by_tid(tid: Tid) {
    if tid == current_thread_id() {
        warn!("Try to kill current Thread[{}], return", tid);
        return;
    }
    if let Some(t) = thread_lookup(tid) {
        if t.privilege() == PrivilegedLevel::Kernel {
            warn!("Try to kill kernel thread[{}], return", tid);
            return;
        }
        thread_destroy(&t);
    } else {
        warn!("Thread [{}] not exist!!!", tid);
    }
}

static CORE_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Wake up target thread.
/// Set its status as Runnable and add it to target cpu's scheduler.
pub fn thread_wake(t: &Thread) {
    let mut status = t.0.inner_mut.status.lock();
    *status = Status::Runnable;

    let affinity_core_id = match t.affinity_core() {
        Some(affinity_core_id) => affinity_core_id,
        None => CORE_COUNTER.fetch_add(1, Ordering::SeqCst) % crate::board::BOARD_CORE_NUMBER,
    };

    let target_cpu = get_cpu(affinity_core_id);
    target_cpu.scheduler().add(t.clone());
    trace!(
        "thread_wake set thread [{}] Runnable on core [{}]",
        t.tid(),
        affinity_core_id
    );
}

/// Wake up target thread by thread id.
/// See thread_wake for more details.
pub fn thread_wake_by_tid(tid: Tid) {
    if tid == current_thread_id() {
        // debug!("Try to wake up running Thread[{}], return", tid);
        return;
    }
    if let Some(t) = thread_lookup(tid) {
        thread_wake(&t);
    } else {
        warn!("Thread [{}] not exist!!!", tid);
    }
}

/// Wake up target thread as the next scheduled thread.
/// Set its status as Runnable and add it to the front of scheduler's queue.
pub fn thread_wake_to_front(t: &Thread) {
    trace!("thread_wake set thread [{}] as next thread", t.tid());
    let mut status = t.0.inner_mut.status.lock();
    *status = Status::Runnable;
    let affinity_core_id = match t.affinity_core() {
        Some(affinity_core_id) => affinity_core_id,
        None => CORE_COUNTER.fetch_add(1, Ordering::SeqCst) % crate::board::BOARD_CORE_NUMBER,
    };
    let target_cpu = get_cpu(affinity_core_id);
    target_cpu.scheduler().add(t.clone());
}

/// Wake up target thread as the next scheduled by thread id.
/// See thread_wake_to_front for more details.
pub fn thread_wake_to_front_by_tid(tid: Tid) {
    if tid == current_thread_id() {
        warn!("Try to wake up running Thread[{}], return", tid);
        return;
    }
    if let Some(t) = thread_lookup(tid) {
        thread_wake_to_front(&t);
    } else {
        warn!("Thread [{}] not exist!!!", tid);
    }
}

/// Block current thread.
/// Set its status as Blocked and it can not scheduled again until waked up.
pub fn thread_block_current() {
    if let Some(current_thread) = cpu().running_thread() {
        irqsave(|| {
            debug!("Thread[{}]  thread_block_current", current_thread.tid());
            let t = &current_thread;
            let reason = Status::Blocked;
            assert_ne!(reason, Status::Runnable);
            let mut status = t.0.inner_mut.status.lock();
            *status = reason;
            drop(status);
        });
    } else {
        warn!("No Running Thread!");
    }
}

/// Block current thread with specific timeout us.
/// Set its status as Blocked and it can not scheduled until blocked time exhausted.
pub fn thread_block_current_with_timeout_us(timeout_us: usize) {
    debug!(
        "Thread[{}]  thread_block_current_with_timeout_us {} microseconds",
        current_thread_id(),
        timeout_us
    );
    if timeout_us >= crate::drivers::timer::TIMER_TICK_US as usize {
        // Enough time to set a wakeup timer and block the current task.
        thread_block_current_with_timeout(timeout_us / 1000)
    } else if timeout_us > 0 {
        // Not enough time to set a wakeup timer, so just do busy-waiting.
        use crate::libs::timer::current_us;
        let end = current_us() + timeout_us;
        while current_us() < end {
            thread_yield()
        }
    }
}

/// Block current thread with specific timeout ms.
/// Set its status as Blocked and it can not scheduled until blocked time exhausted.
pub fn thread_block_current_with_timeout(timeout_ms: usize) {
    if let Some(current_thread) = cpu().running_thread() {
        irqsave(|| {
            warn!(
                "Thread[{}] thread_block_current_with_timeout {} milliseconds",
                current_thread.tid(),
                timeout_ms
            );
            let t = &current_thread;
            let reason = Status::Blocked;
            assert_ne!(reason, Status::Runnable);
            let mut status = t.0.inner_mut.status.lock();
            *status = reason;
            drop(status);
            cpu().scheduler().blocked(t.clone(), Some(timeout_ms));
        });
    } else {
        warn!("No Running Thread!");
    }
}

#[inline(always)]
/// Waits for the associated thread to finish.
pub fn thread_join(id: Tid) {
    if let Some(current_thread) = cpu().running_thread() {
        debug!(
            "Thread [{}] is waiting for thread [{}]",
            current_thread.tid(),
            id
        );
        {
            match THREAD_WAITING_QUEUE.lock().get_mut(&id) {
                Some(queue) => {
                    let t = &current_thread;
                    let reason = Status::Blocked;
                    assert_ne!(reason, Status::Runnable);
                    let mut status = t.0.inner_mut.status.lock();
                    *status = reason;
                    queue.push_back(t.clone());
                }
                None => {
                    return;
                }
            }
        }
        thread_yield();
    } else {
        warn!("No Running Thread!");
    }
}

#[inline(always)]
fn handle_waiting_threads(id: Tid) {
    // wakeup threads which are waiting for thread with the identifier id.
    if let Some(mut queue) = THREAD_WAITING_QUEUE.lock().remove(&id) {
        while let Some(t) = queue.pop_front() {
            thread_wake(&t);
        }
    }
}

/// Regularly wake up blocked threads according to blocked time.
/// This function is called during the process of timer interrupt.
pub fn handle_blocked_threads() {
    use crate::libs::timer::current_ms;
    while let Some(t) = cpu().scheduler().get_wakeup_thread_by_time(current_ms()) {
        debug!("handle_blocked_threads: thread [{}] is wake up", t.tid());
        thread_wake(&t);
    }
}

/// Regularly clean up exited threads.
/// This function is called during the process of timer interrupt.
pub fn handle_exit_threads() {
    let mut exited_thread_queue = thread_exit_queue().lock();
    while let Some(t) = exited_thread_queue.pop_front() {
        thread_destroy(&t);
    }
}

/// Actively give up CPU clock cycles.
/// Todo: make thread yield more efficient.
// #[inline(always)]
pub fn thread_yield() {
    irqsave(|| {
        // debug!("call cpu schedule");
        cpu().schedule();
    });
}

/// Get current running thread id, return 0 if there is no running thread.
pub fn current_thread_id() -> Tid {
    match cpu().running_thread() {
        None => 0,
        Some(t) => t.tid(),
    }
}

/// Get current running thread.
pub fn current_thread() -> Result<Thread, Error> {
    match cpu().running_thread() {
        None => Err(ERROR_INTERNAL),
        Some(t) => Ok(t),
    }
}

/// Actively destroy current running thread.
/// After thread_exit is called, current thread's will be inserted into THREAD_EXIT_QUEUE and be dropped in the future.
/// This function will call `thread_yield` to schedule to next active thread.
pub fn thread_exit() {
    crate::arch::irq::disable();
    let mut t = current_thread().unwrap_or_else(|_| panic!("failed to get current thread"));
    debug!("thread_exit on Thread [{}]", t.tid());

    handle_waiting_threads(t.tid());

    t.set_exited();
    // cpu().set_running_thread(Some(t.clone()));
    // if let Some(current_thread) = cpu().running_thread() {
    //     if t.tid() == current_thread.tid() {
    //         cpu().set_running_thread(None);
    //     }
    // }
    thread_exit_queue().lock().push_back(t);
    thread_yield();
    warn!("thread_exit, should not reach here!!!");
    // crate::arch::irq::enable();
    loop {}
}

/// Main spawn logic.
/// Spawn a new thread with a given entry address.
/// Use "thread_start" as a wrapper, which automatically calls thread_exit when thread is finished.
/// Whether target thread is waked immediately or sleeping is judged by running.
/// Return its thread ID.
fn _inner_spawn(
    func: extern "C" fn(usize),
    arg: usize,
    running: bool,
    privilege: bool,
    name: Option<String>,
    selector: isize,
) -> Tid {
    let mut tid = 0 as Tid;
    irqsave(|| {
        trace!(
            "thread_spawn func: {:x} arg: {} selector [{}]",
            func as usize,
            arg,
            selector
        );

        // Use "thread_start" as a wrapper, which automatically calls thread_exit when thread is finished.
        extern "C" fn thread_start(func: extern "C" fn(usize), arg: usize) -> usize {
            #[cfg(feature = "unwind")]
            {
                const RETRY_MAX: usize = 5;
                let mut i = 0;
                #[cfg(not(feature = "std"))]
                use crate::libs::unwind::catch::catch_unwind;
                #[cfg(feature = "std")]
                use std::panic::catch_unwind;
                loop {
                    i += 1;
                    let r = catch_unwind(|| {
                        func(arg);
                    });
                    match r {
                        Ok(_) => {
                            break;
                        }
                        Err(_) => {
                            info!("retry #{}", i);
                            // Enable interrupt when first enter this thread.
                            // This is awkward, we may need to improve context switch mechanism, see src/arch/switch.rs.
                            crate::arch::irq::enable_and_wait();
                            if i > RETRY_MAX {
                                break;
                            }
                        }
                    }
                }
            }
            #[cfg(not(feature = "unwind"))]
            func(arg);
            thread_exit();
            0
        }

        // Choose affinity core according to selector.
        let affinity_core = if selector < 0 {
            None
        } else {
            if selector > (crate::board::BOARD_CORE_NUMBER - 1).try_into().unwrap() {
                warn!(
                    "try to spawn on nonexistent core {}, board has only {} cores",
                    selector,
                    crate::board::BOARD_CORE_NUMBER
                );
                Some(0)
            } else {
                Some(selector as usize)
            }
        };

        let child_thread = thread_alloc(
            None,
            affinity_core,
            thread_start as usize,
            func as usize,
            arg,
            privilege,
        );
        // If running, set newly allocated thread as Runnable immediately.
        if running {
            thread_wake(&child_thread);
        }
        tid = child_thread.tid();
        // If appointing thread name, insert it into THREAD_NAME_MAP.
        if let Some(name) = name {
            let mut map = THREAD_NAME_MAP.lock();
            map.insert(tid, name);
        }
    });
    tid
}

/// Spawn a new thread with a given entry address.
/// Target thread is waked immediately.
/// Return its thread ID.
pub fn thread_spawn(func: extern "C" fn(usize), arg: usize) -> Tid {
    _inner_spawn(func, arg, true, false, None, -1)
}

/// Spawn a new thread with a given entry address and name.
/// Target thread is waked immediately.
/// Return its thread ID.
pub fn thread_spawn_name(func: extern "C" fn(usize), arg: usize, name: &str) -> Tid {
    _inner_spawn(func, arg, true, false, Some(String::from(name)), -1)
}

/// Spawn a new thread with a given entry address and its name.
/// Target thread is not waked immediately.
/// Return its thread ID.
pub fn thread_spawn_bg(func: extern "C" fn(usize), arg: usize, name: &str) -> Tid {
    _inner_spawn(func, arg, false, false, Some(String::from(name)), -1)
}

/// Spawn a new thread with a given entry address and name.
/// Target thread is privilege, which means it can not be killed.
/// Target thread is waked immediately.
/// Return its thread ID.
#[allow(unused)]
pub(crate) fn thread_spawn_privilege(func: extern "C" fn(usize), arg: usize, name: &str) -> Tid {
    _inner_spawn(func, arg, true, true, Some(String::from(name)), -1)
}

/// Spawn a new thread with a given entry address and core id.
/// Target thread is waked immediately on target core.
/// Return its thread ID.
#[allow(unused)]
pub fn thread_spawn_on_core(func: extern "C" fn(usize), arg: usize, core_id: isize) -> Tid {
    _inner_spawn(func, arg, true, false, None, core_id)
}
