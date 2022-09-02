/// An executor, which is run when idling on network I/O.
use async_task::{Runnable, Task};
use crossbeam_queue::SegQueue;
use futures_lite::pin;
use lazy_static::lazy_static;

use smoltcp::time::{Duration, Instant};

use alloc::{sync::Arc, task::Wake, vec::Vec};
use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};

use super::interface::network_delay;
use crate::drivers::net::set_polling_mode;
use crate::lib::thread::{
    current_thread_id, thread_block_current_with_timeout, thread_wake_by_tid, thread_yield, Tid,
};
use crate::lib::synch::spinlock::Spinlock;
use crate::lib::timer::current_ms;

lazy_static! {
    static ref QUEUE: Spinlock<SegQueue<Runnable>> = Spinlock::new(SegQueue::new());
}

fn run_executor() {
    // println!("run executor, queue len {}", QUEUE.len());
    let queue = QUEUE.lock();
    // let icntr2 = crate::lib::timer::current_cycle();
    // info!("net run executor queue lock cycle {}", icntr2 - icntr);
    let mut runnables: Vec<Runnable> = Vec::with_capacity(queue.len());
    // let icntr = crate::lib::timer::current_cycle();
    while let Some(runnable) = queue.pop() {
        // println!("seg queue pop");
        // runnable.run();
        runnables.push(runnable);
    }
    // let icntr2 = crate::lib::timer::current_cycle();
    // info!("net run executor queue pop cycle {} ", icntr2 - icntr);
    drop(queue);
    for runnable in runnables {
        runnable.run();
    }
}

/// Spawns a future on the executor.
pub fn spawn<F, T>(future: F) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let schedule = |runnable| QUEUE.lock().push(runnable);
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
        let unparked = self.unparked.swap(true, Ordering::AcqRel);
        // info!(
        //     "Thread [{}] wake by ref: unparked {}",
        //     self.thread, unparked
        // );
        if !unparked {
            thread_wake_by_tid(self.thread);
        }
    }
}

#[inline]
pub(crate) fn now() -> Instant {
    Instant::from_millis(current_ms() as i64)
}

/// Blocks the current thread on `f`, running the executor when idling.
pub fn block_on<F, T>(future: F, timeout: Option<Duration>) -> Result<T, ()>
where
    F: Future<Output = T>,
{
    set_polling_mode(true);
    let start = now();
    let thread_notify = Arc::new(ThreadNotify::new());
    let waker = thread_notify.clone().into();
    let mut cx = Context::from_waker(&waker);
    // Pins a variable of type T on the stack and rebinds it as Pin<&mut T>.
    pin!(future);

    loop {
        run_executor();

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

        // Return an advisory wait time for calling [poll] the next time.
        let delay = network_delay(start).map(|d| d.total_millis());

        // debug!("block_on, Poll not Ready, get delay {:?}", delay);

        if delay.unwrap_or(10_000) > 100 {
            let unparked = thread_notify.unparked.swap(false, Ordering::AcqRel);
            // info!(
            //     "block_on() unparked {} delay {:?}",
            //     unparked, delay
            // );
            if !unparked {
                if delay.is_some() {
                    debug!(
                        "block_on() unparked {} delay {:?}",
                        unparked, delay
                    );
                    thread_block_current_with_timeout(delay.unwrap() as usize);
                }
                // allow interrupts => NIC thread is able to run
                set_polling_mode(false);
                // switch to another task
                thread_yield();
                // Polling mode => no NIC interrupts => NIC thread should not run
                set_polling_mode(true);
            }
        }
    }
    // })
}
