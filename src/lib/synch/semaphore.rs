use crate::lib::thread::{current_thread, thread_block_current, thread_wake_to_front, Thread, thread_yield};
use alloc::collections::VecDeque;

use super::spinlock::SpinlockIrqSave;

struct SemaphoreState {
    value: usize,
    queue: Option<VecDeque<Thread>>,
}

pub struct Semaphore {
    inner: SpinlockIrqSave<SemaphoreState>,
}

pub enum SemaphoreWaitResult {
    Acquired,
    Enqueued,
}

// Same unsafe impls as `Semaphore`
unsafe impl Sync for Semaphore {}
unsafe impl Send for Semaphore {}

impl Semaphore {
    pub const fn new(value: usize) -> Self {
        Semaphore {
            inner: SpinlockIrqSave::new(SemaphoreState { value, queue: None }),
        }
    }

    /// Acquires a resource of this semaphore, blocking the current thread until
    /// it can do so or until the wakeup time (in ms) has elapsed.
    ///
    /// This method will block until the internal count of the semaphore is at
    /// least 1.
    pub fn acquire(&self) {

        // Loop until we have acquired the semaphore.
        loop {
            trace!("acquire loop");
            match current_thread() {
                Ok(t) => {
                    let mut inner = self.inner.lock();
                    if inner.value == 0 {
                        thread_block_current();
                        if let Some(queue) = &mut inner.queue {
                            queue.push_back(t.clone());
                        } else {
                            let mut queue = VecDeque::new();
                            queue.push_back(t.clone());
                            inner.queue = Some(queue);
                        }
                        /* Before yield, we need to drop the lock. */
                        drop(inner);
                        thread_yield();
                        trace!("return to this thread");
                    } else {
                        // Successfully acquired the semaphore.
                        inner.value -= 1;
                        trace!("semaphore acquire success, current value {}, return", inner.value);
                        return;
                    }
                }
                Err(_) => {
                    error!("failed to get current_thread");
                    return;
                }
            }
        }
    }

    /// Release a resource from this semaphore.
    ///
    /// This will increment the number of resources in this semaphore by 1 and
    /// will notify any pending waiters in `acquire` or `access` if necessary.
    pub fn release(&self) {
        let mut inner = self.inner.lock();
        // debug!(
        //     "semaphore release on thread [{}], value from {} to ({})",
        //     crate::lib::thread::current_thread().unwrap().tid(),
        //     inner.value,
        //     inner.value + 1
        // );
        inner.value += 1;
        if let Some(queue) = &mut inner.queue {
            if let Some(t) = queue.pop_front() {
                /* Before yield, we need to drop the lock. */
                drop(inner);
                thread_wake_to_front(&t);
            }
        }
    }
}
