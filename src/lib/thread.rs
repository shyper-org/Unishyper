use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;

use spin::Mutex;

use crate::arch::{ContextFrame, PAGE_SIZE, STACK_SIZE};
use crate::lib::cpu::cpu;
use crate::lib::error::*;
use crate::lib::scheduler::scheduler;
use crate::lib::traits::*;
use crate::mm::{Addr, Region};
use crate::util::round_up;

pub type Tid = usize;

#[derive(Debug)]
pub enum PrivilegedLevel {
    User,
    Kernel,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Status {
    Runnable,
    Sleep,
    Blocked,
    WaitForEvent,
    WaitForReply,
    WaitForRequest,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Inner {
    uuid: usize,
    parent: Option<usize>,
    level: PrivilegedLevel,
    stack: Region,
}

struct InnerMut {
    status: Mutex<Status>,
    context_frame: Mutex<ContextFrame>,
    mem_regions: Mutex<BTreeMap<Addr, Region>>,
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
    pub fn tid(&self) -> Tid {
        self.0.inner.uuid
    }

    pub fn parent(&self) -> Option<Tid> {
        self.0.inner.parent
    }

    pub fn is_child_of(&self, tid: Tid) -> bool {
        match &self.0.inner.parent {
            None => false,
            Some(t) => *t == tid,
        }
    }

    pub fn runnable(&self) -> bool {
        let lock = self.0.inner_mut.status.lock();
        *lock == Status::Runnable
    }

    pub fn wait_for_reply<F>(&self, f: F) -> bool
    where
        F: FnOnce(),
    {
        let mut status = self.0.inner_mut.status.lock();
        if *status == Status::WaitForReply {
            f();
            *status = Status::Runnable;
            scheduler().add_front(self.clone());
            true
        } else {
            false
        }
    }

    pub fn wait_for_request<F>(&self, f: F) -> bool
    where
        F: FnOnce(),
    {
        let mut status = self.0.inner_mut.status.lock();
        if *status == Status::WaitForRequest {
            f();
            *status = Status::Runnable;
            scheduler().add_front(self.clone());
            true
        } else {
            false
        }
    }

    pub fn set_context(&self, ctx: ContextFrame) {
        let mut context_frame = self.0.inner_mut.context_frame.lock();
        *context_frame = ctx;
    }

    pub fn context(&self) -> ContextFrame {
        let lock = self.0.inner_mut.context_frame.lock();
        lock.clone()
    }

    pub fn map_with_context<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut ContextFrame) -> T,
    {
        let mut context_frame = self.0.inner_mut.context_frame.lock();
        f(&mut *context_frame)
    }

    pub fn add_address_space(&self, addr: Addr, region: Region) {
        let mut addr_space = self.0.inner_mut.mem_regions.lock();
        addr_space.insert(addr, region);
    }

    pub fn free_address_space(&self, addr: Addr) {
        let mut addr_space = self.0.inner_mut.mem_regions.lock();
        addr_space.remove(&addr);
    }
}

static THREAD_UUID_ALLOCATOR: AtomicUsize = AtomicUsize::new(100);

fn new_tid() -> Tid {
    THREAD_UUID_ALLOCATOR.fetch_add(1, Relaxed)
}

static THREAD_MAP: Mutex<BTreeMap<Tid, Thread>> = Mutex::new(BTreeMap::new());

pub fn thread_alloc2(pc: usize, arg0: usize, arg1: usize) -> Thread {
    let id = new_tid();

    let stack_size = round_up(STACK_SIZE, PAGE_SIZE);
    let stack_region = crate::mm::page_pool::pages_alloc(stack_size / PAGE_SIZE)
        .expect("fail to allocate user thread stack");
    let stack_start = stack_region.kva();

    let sp = stack_start + stack_region.size();

    let t = Thread(Arc::new(ControlBlock {
        inner: Inner {
            uuid: id,
            parent: None,
            level: PrivilegedLevel::Kernel,
            stack: stack_region,
        },
        inner_mut: InnerMut {
            status: Mutex::new(Status::Sleep),
            context_frame: Mutex::new(ContextFrame::new(pc, sp, arg0, arg1, true)),
            mem_regions: Mutex::new(BTreeMap::new()),
        },
    }));
    let mut map = THREAD_MAP.lock();
    map.insert(id, t.clone());

    debug!(
        "thread_alloc success id [{}] sp [{:x} to {:x}]",
        id,
        stack_start,
        sp
    );
    t
}

pub fn thread_alloc(pc: usize, arg: usize) -> Thread {
    thread_alloc2(pc, arg, 0)
}

pub fn thread_lookup(tid: Tid) -> Option<Thread> {
    let map = THREAD_MAP.lock();
    map.get(&tid).cloned()
}

pub fn thread_destroy(t: Thread) {
    debug!("Destroy t{}", t.tid());
    if let Some(current_thread) = crate::lib::cpu::cpu().running_thread() {
        if t.tid() == current_thread.tid() {
            crate::lib::cpu::cpu().set_running_thread(None);
        }
    }
    let mut map = THREAD_MAP.lock();
    map.remove(&t.tid());
}

pub fn thread_wake(t: &Thread) {
    debug!("thread_wake set thread [{}] Runnable", t.tid());
    let mut status = t.0.inner_mut.status.lock();
    *status = Status::Runnable;
    scheduler().add(t.clone());
}

pub fn thread_wake_by_tid(tid: Tid) {
    if let Some(t) = thread_lookup(tid) {
        thread_wake(&t);
    } else {
        warn!("Thread{} not exist!!!", tid);
    }
}

pub fn thread_wake_to_front(t: &Thread) {
    trace!("thread_wake set thread [{}] as next thread", t.tid());
    let mut status = t.0.inner_mut.status.lock();
    *status = Status::Runnable;
    scheduler().add_front(t.clone());
}

pub fn thread_block_current() {
    if let Some(current_thread) = crate::lib::cpu::cpu().running_thread() {
        debug!("Thread[{}]  thread_block_current", current_thread.tid());
        let t = &current_thread;
        let reason = Status::Blocked;
        assert_ne!(reason, Status::Runnable);
        let mut status = t.0.inner_mut.status.lock();
        *status = reason;
        drop(status);
    } else {
        warn!("No Running Thread!");
    }
}

pub fn thread_block_current_with_timeout(timeout: usize) {
    if let Some(current_thread) = crate::lib::cpu::cpu().running_thread() {
        debug!(
            "Thread[{}] thread_block_current_with_timeout {}",
            current_thread.tid(),
            timeout
        );
        let t = &current_thread;
        let reason = Status::Blocked;
        assert_ne!(reason, Status::Runnable);
        let mut status = t.0.inner_mut.status.lock();
        *status = reason;
        drop(status);
        scheduler().blocked(t.clone(), Some(timeout));
    } else {
        warn!("No Running Thread!");
    }
}

pub fn handle_blocked_threads() {
    use crate::lib::timer::current_ms;
    while let Some(t) = scheduler().get_wakeup_thread_by_time(current_ms()) {
        thread_wake(&t);
    }
}

// Todo: make thread yield more efficient.
#[no_mangle]
pub fn thread_yield() {
    // let icntr = crate::lib::timer::current_cycle();
    trace!(
        "thread_yield is called on Thread [{}]",
        get_current_thread_id()
    );
    crate::arch::switch_to();
    // let icntr2 = crate::lib::timer::current_cycle();
    // info!("as create cycle {}", icntr2 - icntr);
}

#[no_mangle]
pub fn thread_schedule() {
    // trace!("thread_schedule\n");
    cpu().schedule();
    // trace!("thread_schedule end\n");
}

pub fn get_current_thread_id() -> Tid {
    match cpu().running_thread() {
        None => 0,
        Some(t) => t.tid(),
    }
}

pub fn current_thread() -> Result<Thread, Error> {
    match cpu().running_thread() {
        None => Err(ERROR_INTERNAL),
        Some(t) => Ok(t),
    }
}
