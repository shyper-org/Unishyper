/// An executor, which is run when idling on network I/O.
use async_task::{Runnable, Task};
use futures_lite::pin;

use smoltcp::time::Duration;

use alloc::{sync::Arc, task::Wake, vec::Vec};
use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};

use crate::libs::thread::{current_thread_id, thread_wake_by_tid, Tid};
use crate::libs::thread::{thread_block_current_with_timeout, thread_yield};

use crate::libs::synch::spinlock::Spinlock;
use crate::libs::error::ShyperError;

static QUEUE: Spinlock<Vec<Runnable>> = Spinlock::new(Vec::new());

pub fn network_delay(timestamp: smoltcp::time::Instant) -> Option<Duration> {
    crate::libs::net::NIC
        .lock()
        .as_nic_mut()
        .unwrap()
        .poll_delay(timestamp)
}

fn run_executor() {
    let mut queue = QUEUE.lock();
    // println!("run executor, queue len {}", queue.len());
    let mut runnables: Vec<Runnable> = Vec::with_capacity(queue.len());
    while let Some(runnable) = queue.pop() {
        // println!("seg queue pop");
        // runnable.run();
        runnables.push(runnable);
    }
    drop(queue);
    for runnable in runnables {
        runnable.run();
    }
}

fn executor_set_polling_mode(value: bool) {
    let _ = crate::drivers::get_network_driver()
        .unwrap()
        .lock()
        .set_polling_mode(value);
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
        trace!("Thread {} Dropping ThreadNotify!", self.thread);
    }
}

impl Wake for ThreadNotify {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        // Make sure the wakeup is remembered until the next `park()`.
        let unparked = self.unparked.swap(true, Ordering::AcqRel);
        debug!("Thread {} wake by ref: unparked {}", self.thread, unparked);
        if !unparked {
            crate::util::irqsave(|| {
                thread_wake_by_tid(self.thread);
            });
        }
    }
}

/// Blocks the current thread on `f`, running the executor when idling.
pub fn block_on<F, T>(future: F, _timeout: Option<Duration>) -> Result<T, ShyperError>
where
    F: Future<Output = T>,
{
    // debug!("block_on is called");
    // Polling mode => no NIC interrupts => NIC thread should not run
    executor_set_polling_mode(true);

    let start = super::now();

    let thread_notify = Arc::new(ThreadNotify::new());
    let waker = thread_notify.clone().into();
    let mut cx = Context::from_waker(&waker);
    // Pins a variable of type T on the stack and rebinds it as Pin<&mut T>.A
    pin!(future);

    loop {
        // run background tasks
        run_executor();

        // let now = now();

        if let Poll::Ready(t) = future.as_mut().poll(&mut cx) {
            // if let Some(delay_millis) = network_delay(now()).map(|d| d.total_millis()) {
            //     debug!(
            //         "block_on() first poll start {} delay_millis {}",
            //         start.millis(),
            //         delay_millis
            //     );
            //     thread_block_current_with_timeout(delay_millis as usize);
            // }

            // allow interrupts => NIC thread is able to run
            executor_set_polling_mode(false);
            // debug!("block on return ok");
            return Ok(t);
        }

        if let Some(duration) = _timeout {
            if super::now() >= start + duration {
                if let Some(delay_millis) = network_delay(super::now()).map(|d| d.total_millis()) {
                    thread_block_current_with_timeout(delay_millis as usize);
                }

                // allow interrupts => NIC thread is able to run
                executor_set_polling_mode(false);
                debug!("block on return err");
                return Err(ShyperError::BadState);
            }
        }

        // Return an advisory wait time for calling [poll] the next time.
        let delay = network_delay(super::now()).map(|d| d.total_millis());

        if delay.unwrap_or(10_000) > 100 {
            let unparked = thread_notify.unparked.swap(false, Ordering::AcqRel);
            if !unparked {
                if delay.is_some() {
                    thread_block_current_with_timeout(delay.unwrap() as usize);
                }
                // allow interrupts => NIC thread is able to run
                executor_set_polling_mode(false);
                // switch to another task
                thread_yield();
                // Polling mode => no NIC interrupts => NIC thread should not run
                executor_set_polling_mode(true);
            }
        }
    }
}
