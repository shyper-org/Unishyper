/// An executor, which is run when idling on network I/O.
use async_task::{Runnable, Task};
use crossbeam_queue::SegQueue;
use futures_lite::pin;
use lazy_static::lazy_static;

use smoltcp::time::{Duration, Instant};

use spin::Mutex;

use alloc::{collections::BTreeMap, sync::Arc, task::Wake, vec::Vec};
use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, Waker},
};

use super::interface::network_delay;
use crate::drivers::net::set_polling_mode;
use crate::lib::thread::{
    current_thread_id, thread_block_current_with_timeout, thread_wake_by_tid,
    thread_yield, Tid,
};
use crate::lib::timer::current_ms;

lazy_static! {
    static ref QUEUE: SegQueue<Runnable> = SegQueue::new();
}

fn run_executor() {
    // println!("run executor, queue len {}", QUEUE.len());
    let mut wake_buf: Vec<Waker> = Vec::with_capacity(QUEUE.len());
    while let Some(runnable) = QUEUE.pop() {
        // println!("seg queue pop");
        // runnable.run();
        wake_buf.push(runnable.waker());
        runnable.run();
    }
    for waker in wake_buf {
        waker.wake();
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
            thread: current_thread_id(),
            unparked: AtomicBool::new(false),
        }
    }
}

impl Drop for ThreadNotify {
    fn drop(&mut self) {
        debug!("Thread {} Dropping ThreadNotify!", self.thread);
    }
}

impl Wake for ThreadNotify {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        // Make sure the wakeup is remembered until the next `park()`.
        let unparked = self.unparked.swap(true, Ordering::Relaxed);
        // println!(
        //     "Thread [{}] wake by ref: unparked {}",
        //     self.thread, unparked
        // );
        if !unparked {
            thread_wake_by_tid(self.thread);
        }
    }
}

// Todo: replace these using thread_local or local_key model.
// thread_local! {
// 	static CURRENT_THREAD_NOTIFY: Arc<ThreadNotify> = Arc::new(ThreadNotify::new());
// }
static CURRENT_THREAD_NOTIFY_MAP: Mutex<BTreeMap<Tid, Arc<ThreadNotify>>> =
    Mutex::new(BTreeMap::new());

fn current_thread_notify() -> Arc<ThreadNotify> {
    let mut map = CURRENT_THREAD_NOTIFY_MAP.lock();
    let cur_tid = &current_thread_id();

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
    let thread_notify = current_thread_notify();

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
    let thread_notify = current_thread_notify();

    set_polling_mode(true);
    let start = Instant::from_millis(current_ms() as i64);
    let waker = thread_notify.clone().into();
    let mut cx = Context::from_waker(&waker);
    // Pins a variable of type T on the stack and rebinds it as Pin<&mut T>.
    pin!(future);

    loop {
        if let Poll::Ready(t) = future.as_mut().poll(&mut cx) {
            // println!("block_on, Poll::Ready");
            set_polling_mode(false);
            return Ok(t);
        }

        // println!("block_on, Poll not Ready");

        if let Some(duration) = timeout {
            if Instant::from_millis(current_ms() as i64) >= start + duration {
                set_polling_mode(false);
                return Err(());
            }
        }

        run_executor();

        // Return an advisory wait time for calling [poll] the next time.
        let delay = network_delay(start).map(|d| d.total_millis());

        // debug!("block_on, Poll not Ready, get delay {:?}", delay);

        match delay {
            Some(d) => {
                if d > 100 {
                    let unparked = thread_notify.unparked.swap(false, Ordering::Acquire);
                    // println!("block_on, Poll not Ready, unparked {}", unparked);
                    if !unparked {
                        set_polling_mode(false);
                        thread_block_current_with_timeout(d as usize);
                        thread_yield();
                        // info!("yield return");
                        set_polling_mode(true);
                        thread_notify.unparked.store(false, Ordering::Release);
                    }
                }
            }
            None => {}
        }
    }
    // })
}
