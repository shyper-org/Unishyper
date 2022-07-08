/// An executor, which is run when idling on network I/O.
use async_task::{Runnable, Task};
use crossbeam_queue::SegQueue;
use futures_lite::pin;
use lazy_static::lazy_static;

use smoltcp::time::{Duration, Instant};

use spin::Mutex;

use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};

use super::interface::network_delay;

use crate::lib::thread::{
    get_current_thread_id, thread_block_current, thread_block_current_with_timeout,
    thread_wake_by_tid, thread_yield,
};
use crate::lib::timer::current_ms;

use crate::drivers::net::set_polling_mode;
/// A thread handle type
// type Tid = u32;
use crate::lib::thread::Tid;

lazy_static! {
    static ref QUEUE: SegQueue<Runnable> = SegQueue::new();
}

fn run_executor() {
    // trace!("run executor");
    while let Some(runnable) = QUEUE.pop() {
        runnable.run();
    }
}

/// Spawns a future on the executor.
pub fn spawn<F, T>(future: F) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let schedule = |runnable| QUEUE.push(runnable);
    let (runnable, task) = async_task::spawn(future, schedule);
    runnable.schedule();
    task
}

struct ThreadNotify {
    /// The (single) executor thread.
    thread: Tid,
    /// A flag to ensure a wakeup is not "forgotten" before the next `block_current_task`
    unparked: AtomicBool,
}

impl ThreadNotify {
    pub fn new() -> Self {
        Self {
            thread: get_current_thread_id(),
            unparked: AtomicBool::new(false),
        }
    }
}

impl Drop for ThreadNotify {
    fn drop(&mut self) {
        info!("Thread {} Dropping ThreadNotify!", self.thread);
    }
}

impl Wake for ThreadNotify {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        // Make sure the wakeup is remembered until the next `park()`.
        let unparked = self.unparked.swap(true, Ordering::Relaxed);
        if !unparked {
            thread_wake_by_tid(self.thread);
        }
    }
}

// thread_local! {
// 	static CURRENT_THREAD_NOTIFY: Arc<ThreadNotify> = Arc::new(ThreadNotify::new());
// }

// Todo: replace these using thread_local or local_key model.
static CURRENT_THREAD_NOTIFY_MAP: Mutex<BTreeMap<Tid, Arc<ThreadNotify>>> =
    Mutex::new(BTreeMap::new());

fn get_current_thread_notify() -> Arc<ThreadNotify> {
    let mut map = CURRENT_THREAD_NOTIFY_MAP.lock();
    let cur_tid = &get_current_thread_id();

    if !map.contains_key(cur_tid) {
        let arc_thread_notify = Arc::new(ThreadNotify::new());
        map.insert(cur_tid.clone(), arc_thread_notify.clone());
        return arc_thread_notify.clone();
    } else {
        return map.get(cur_tid).unwrap().clone();
    }
}

pub fn poll_on<F, T>(future: F, timeout: Option<Duration>) -> Result<T, ()>
where
    F: Future<Output = T>,
{
    // CURRENT_THREAD_NOTIFY.with(|thread_notify| {
    let thread_notify = get_current_thread_notify();

    set_polling_mode(true);

    let start = Instant::from_millis(current_ms() as i64);
    let waker = &thread_notify.clone().into();
    let mut cx = Context::from_waker(&waker);
    pin!(future);

    loop {
        if let Poll::Ready(t) = future.as_mut().poll(&mut cx) {
            set_polling_mode(false);
            return Ok(t);
        }

        if let Some(duration) = timeout {
            if Instant::from_millis(current_ms() as i64) >= start + duration {
                set_polling_mode(false);
                return Err(());
            }
        }

        run_executor()
    }
    // })
}

/// Blocks the current thread on `f`, running the executor when idling.
pub fn block_on<F, T>(future: F, timeout: Option<Duration>) -> Result<T, ()>
where
    F: Future<Output = T>,
{
    // CURRENT_THREAD_NOTIFY.with(|thread_notify| {
    trace!("block_on");
    let thread_notify = get_current_thread_notify();

    let start = Instant::from_millis(current_ms() as i64);
    let waker = thread_notify.clone().into();
    let mut cx = Context::from_waker(&waker);
    pin!(future);

    loop {
        if let Poll::Ready(t) = future.as_mut().poll(&mut cx) {
            trace!("block_on, Poll::Ready");
            return Ok(t);
        }

        // trace!("block_on, Poll not Ready");

        if let Some(duration) = timeout {
            if Instant::from_millis(current_ms() as i64) >= start + duration {
                return Err(());
            }
        }

        // Return an advisory wait time for calling [poll] the next time.
        let delay =
            network_delay(Instant::from_millis(current_ms() as i64)).map(|d| d.total_millis());

        debug!("block_on, Poll not Ready, get delay {:?}", delay);

        if delay.is_none() || delay.unwrap() > 100 {
            let unparked = thread_notify.unparked.swap(false, Ordering::Acquire);
            // trace!("block_on, Poll not Ready, unparked {}", unparked);
            if !unparked {
                match delay {
                    Some(d) => thread_block_current_with_timeout(d),
                    None => thread_block_current(),
                };
                thread_yield();
                thread_notify.unparked.store(false, Ordering::Release);
                run_executor()
            }
        } else {
            // trace!("block_on, Poll not Ready, delay or delay < 100");
            run_executor()
        }
    }
    // })
}
