use crate::lib::thread::{current_thread, thread_sleep, thread_wake, Thread};
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
        loop {
            match current_thread() {
                Ok(t) => {
                    let mut inner = self.inner.lock();
                    if inner.value == 0 {
                        thread_sleep(&t, crate::lib::thread::Status::Blocked);
                        if let Some(queue) = &mut inner.queue {
                            queue.push_back(t);
                        } else {
                            let mut queue = VecDeque::new();
                            queue.push_back(t);
                            inner.queue = Some(queue);
                        }
                    } else {
                        inner.value -= 1;
                        return;
                    }
                }
                Err(_) => {
                    debug!("failed to get current_thread");
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
        if inner.value != 0 {
            inner.value += 1;
        } else {
            if let Some(queue) = &mut inner.queue {
                if let Some(t) = queue.pop_front() {
                    thread_wake(&t);
                    crate::lib::cpu::cpu().schedule();
                }
            }
        }
    }
}
