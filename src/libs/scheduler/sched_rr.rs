use spin::Mutex;

use alloc::collections::{BTreeMap, VecDeque};

use crate::libs::{thread::Thread, timer::current_ms};

use super::Scheduler;

pub struct RoundRobinScheduler {
    // ready_queue: Mutex<VecDeque<Thread>>,
    running_queue: Mutex<VecDeque<Thread>>,
    blocked_queue: Mutex<BTreeMap<usize, Thread>>,
}

impl RoundRobinScheduler {
    pub fn new() -> Self {
        RoundRobinScheduler {
            running_queue: Mutex::new(VecDeque::new()),
            blocked_queue: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn show_running_threads(&self) {
        for t in self.running_queue.lock().iter() {
            debug!("Running Thread[{}]", t.tid());
        }
    }

    pub fn show_blocked_threads(&self) {
        for t in self.blocked_queue.lock().iter() {
            debug!("Blocked Thread[{}], sleep time {}ms", t.1.tid(), t.0);
        }
    }
}

impl Scheduler for RoundRobinScheduler {
    fn add_front(&self, thread: Thread) {
        self.running_queue.lock().push_front(thread);
    }

    fn add(&self, thread: Thread) {
        self.running_queue.lock().push_back(thread);
    }

    fn pop(&self) -> Option<Thread> {
        // for t in self.running_queue.lock().clone().into_iter() {
        //     println!("running queue: thread [{}]", t.tid());
        // }
        self.running_queue.lock().pop_front()
    }

    // Todo: replace timeout with wakeup time.
    fn blocked(&self, thread: Thread, timeout: Option<usize>) {
        let wakeup_time = timeout.map(|t| current_ms() + t);
        debug!(
            "Thread[{}] blocked, timeout: {:?} wakeup_time: {:?}",
            thread.tid(),
            timeout,
            wakeup_time
        );
        if let Some(wt) = wakeup_time {
            self.blocked_queue.lock().insert(wt, thread);
        } else {
            self.blocked_queue.lock().insert(usize::MAX, thread);
        }
    }

    fn get_wakeup_thread_by_time(&self, current_ms: usize) -> Option<Thread> {
        let mut lock = self.blocked_queue.lock();
        if let Some((nearest_wakeup_time, nearest_wakeup_thread)) = lock.first_key_value() {
            if *nearest_wakeup_time < current_ms {
                debug!(
                    "Thread[{}] is removed from blocked queue, wakeuptime: {} current time: {}",
                    nearest_wakeup_thread.tid(),
                    nearest_wakeup_time,
                    current_ms
                );
                let wake_thread = lock.pop_first().unwrap().1;
                return Some(wake_thread);
            }
            // debug!(
            //     "Thread[{}] is first on blocked queue, wakeuptime: {} current time: {}",
            //     nearest_wakeup_thread.tid(),
            //     nearest_wakeup_time,
            //     current_ms
            // );
        }
        return None;
    }
}
