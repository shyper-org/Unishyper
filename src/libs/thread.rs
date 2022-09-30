use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use core::fmt;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;

use spin::Mutex;

use crate::arch::{ContextFrame, PAGE_SIZE, STACK_SIZE};
use crate::libs::cpu::cpu;
use crate::libs::error::*;
use crate::libs::scheduler::scheduler;
use crate::libs::traits::*;
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
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Status::Runnable => write!(f, "Running"),
            Status::Ready => write!(f, "Ready"),
            Status::Blocked => write!(f, "Blocked"),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Inner {
    uuid: usize,
    level: PrivilegedLevel,
    stack: Stack,
}

struct InnerMut {
    status: Mutex<Status>,
    context_frame: Mutex<ContextFrame>,
    mem_regions: Mutex<BTreeMap<VAddr, MappedRegion>>,
}

struct ControlBlock {
    inner: Inner,
    inner_mut: InnerMut,
}

impl Drop for ControlBlock {
    fn drop(&mut self) {
        debug!("Drop t{}", self.inner.uuid);
    }
}

#[derive(Clone)]
pub struct Thread(Arc<ControlBlock>);

impl Thread {
    /// Get thread tid, which is globally unique.
    pub fn tid(&self) -> Tid {
        self.0.inner.uuid
    }

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

    /// Get thread privilege level.
    pub fn privilege(&self) -> PrivilegedLevel {
        self.0.inner.level
    }

    /// Set thread's context_frame.
    pub fn set_context(&self, ctx: ContextFrame) {
        let mut context_frame = self.0.inner_mut.context_frame.lock();
        *context_frame = ctx;
    }

    /// Get thread's context_frame.
    pub fn context(&self) -> ContextFrame {
        let lock = self.0.inner_mut.context_frame.lock();
        lock.clone()
    }

    // Executed something withtin given context, currently not supported.
    // pub fn map_with_context<F, T>(&self, f: F) -> T
    // where
    //     F: FnOnce(&mut ContextFrame) -> T,
    // {
    //     let mut context_frame = self.0.inner_mut.context_frame.lock();
    //     f(&mut *context_frame)
    // }

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
}

static THREAD_UUID_ALLOCATOR: AtomicUsize = AtomicUsize::new(100);

/// Alloc global unique id for new thread.
fn new_tid() -> Tid {
    THREAD_UUID_ALLOCATOR.fetch_add(1, Relaxed)
}

/// Store thread IDs and its corresponding thread struct.
static THREAD_MAP: Mutex<BTreeMap<Tid, Thread>> = Mutex::new(BTreeMap::new());

/// Store thread IDs and its corresponding thread name.
/// It doesn't store all threads' information, cause not all thread have their name.
/// Generally only threads on the background may exist here.
static THREAD_NAME_MAP: Mutex<BTreeMap<Tid, String>> = Mutex::new(BTreeMap::new());

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
/// 1. generate new thread id;
/// 2. alloc mapped memory region for stack according to stack size;
/// 3. construct thread control block, including inner and inner_mut;
/// 4. insert thread struct into glocal THREAD_MAP;
/// 5. return the generated Thread struct.
///
/// Notes: the generated thread is at Ready state, you need to wake it up.
pub fn thread_alloc2(pc: usize, arg0: usize, arg1: usize, privilege: bool) -> Thread {
    let id = new_tid();

    // pub const STACK_SIZE: usize = 32_768; // PAGE_SIZE * 8
    let stack_size = round_up(STACK_SIZE, PAGE_SIZE);

    let stack_region = crate::mm::stack::alloc_stack(stack_size / PAGE_SIZE)
        .expect("fail to allocate user thread stack");
    let stack_start = stack_region.start_address();

    let sp = stack_start + stack_region.size_in_bytes();
    let sp = sp.value();

    let t = Thread(Arc::new(ControlBlock {
        inner: Inner {
            uuid: id,
            level: if privilege {
                PrivilegedLevel::Kernel
            } else {
                PrivilegedLevel::User
            },
            stack: stack_region,
        },
        inner_mut: InnerMut {
            status: Mutex::new(Status::Ready),
            context_frame: Mutex::new(ContextFrame::new(pc, sp, arg0, arg1)),
            mem_regions: Mutex::new(BTreeMap::new()),
        },
    }));
    let mut map = THREAD_MAP.lock();
    map.insert(id, t.clone());

    debug!(
        "thread_alloc success id [{}] sp [{} to 0x{:016x}]",
        id, stack_start, sp
    );
    t
}

/// Thread alloc logic without another arg.
/// See thread_alloc2 for more details.
pub fn thread_alloc(pc: usize, arg: usize, privilege: bool) -> Thread {
    thread_alloc2(pc, arg, 0, privilege)
}

/// Find target thread by thread id.
/// Return None if thread not exist.
pub fn thread_lookup(tid: Tid) -> Option<Thread> {
    let map = THREAD_MAP.lock();
    map.get(&tid).cloned()
}

/// Destory target thread.
/// Remove it from THREAD_NAME_MAP and THREAD_MAP.
pub fn thread_destroy(t: &Thread) {
    debug!("Destroy t{}", t.tid());
    if let Some(current_thread) = crate::libs::cpu::cpu().running_thread() {
        if t.tid() == current_thread.tid() {
            crate::libs::cpu::cpu().set_running_thread(None);
        }
    }
    let mut name_map = THREAD_NAME_MAP.lock();
    name_map.remove(&t.tid());
    let mut map = THREAD_MAP.lock();
    map.remove(&t.tid());
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

/// Wake up target thread.
/// Set its status as Runnable and add it to scheduler().
pub fn thread_wake(t: &Thread) {
    debug!("thread_wake set thread [{}] Runnable", t.tid());
    let mut status = t.0.inner_mut.status.lock();
    *status = Status::Runnable;
    scheduler().add(t.clone());
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
    scheduler().add_front(t.clone());
}

/// Block current thread.
/// Set its status as Blocked and it can not scheduled again until waked up.
pub fn thread_block_current() {
    if let Some(current_thread) = crate::libs::cpu::cpu().running_thread() {
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

/// Block current thread with specific timeout ms.
/// Set its status as Blocked and it can not scheduled until blocked time exhausted.
pub fn thread_block_current_with_timeout(timeout_ms: usize) {
    if let Some(current_thread) = crate::libs::cpu::cpu().running_thread() {
        irqsave(|| {
            debug!(
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
            scheduler().blocked(t.clone(), Some(timeout_ms));
        });
    } else {
        warn!("No Running Thread!");
    }
}

/// Regularly wake up blocked threads according to blocked time.
/// This function is called during the process of timer interrupt.
pub fn handle_blocked_threads() {
    use crate::libs::timer::current_ms;
    while let Some(t) = scheduler().get_wakeup_thread_by_time(current_ms()) {
        debug!("handle_blocked_threads: thread [{}] is wake up", t.tid());
        thread_wake(&t);
    }
}

/// Actively give up CPU clock cycles.
/// Todo: make thread yield more efficient.
#[no_mangle]
pub fn thread_yield() {
    // debug!("thread_yield is called on Thread [{}]", current_thread_id());
    crate::arch::switch_to();
}

#[no_mangle]
/// Call cpu scheduler to schedule to next thread.
pub fn thread_schedule() {
    // trace!("thread_schedule\n");
    cpu().schedule();
    // trace!("thread_schedule end\n");
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

/// Actively destory current running thread.
pub fn thread_exit() {
    let result = current_thread();
    match result {
        Ok(t) => {
            crate::libs::thread::thread_destroy(&t);
        }
        Err(_) => {
            panic!("failed to get current_thread");
        }
    }
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
) -> Tid {
    let mut tid = 0 as Tid;
    irqsave(|| {
        debug!("thread_spawn func: {:x} arg: {}", func as usize, arg);

        // Use "thread_start" as a wrapper, which automatically calls thread_exit when thread is finished.
        extern "C" fn thread_start(func: extern "C" fn(usize), arg: usize) -> usize {
            func(arg);
            thread_exit();
            0
        }

        let child_thread = thread_alloc2(thread_start as usize, func as usize, arg, privilege);
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
    _inner_spawn(func, arg, true, false, None)
}

/// Spawn a new thread with a given entry address and name.
/// Target thread is waked immediately.
/// Return its thread ID.
pub fn thread_spawn_name(func: extern "C" fn(usize), arg: usize, name: &str) -> Tid {
    _inner_spawn(func, arg, true, false, Some(String::from(name)))
}

/// Spawn a new thread with a given entry address and its name.
/// Target thread is not waked immediately.
/// Return its thread ID.
pub fn thread_spawn_bg(func: extern "C" fn(usize), arg: usize, name: &str) -> Tid {
    _inner_spawn(func, arg, false, false, Some(String::from(name)))
}

/// Spawn a new thread with a given entry address and name.
/// Target thread is privilege, which means it can not be killed.
/// Target thread is waked immediately.
/// Return its thread ID.
#[allow(unused)]
pub(crate) fn thread_spawn_privilege(func: extern "C" fn(usize), arg: usize, name: &str) -> Tid {
    _inner_spawn(func, arg, true, true, Some(String::from(name)))
}
