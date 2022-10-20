use alloc::collections::{BTreeMap, VecDeque};

use spin::{Mutex, Once};

use crate::libs::{thread::Thread, timer::current_ms};

pub struct RoundRobinScheduler {
    running_queue: Mutex<VecDeque<Thread>>,
    blocked_queue: Mutex<BTreeMap<usize, Thread>>,
}

impl RoundRobinScheduler {
    fn new() -> Self {
        RoundRobinScheduler {
            running_queue: Mutex::new(VecDeque::new()),
            blocked_queue: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn add_front(&self, thread: Thread) {
        let mut running_queue = self.running_queue.lock();
        running_queue.push_front(thread);
    }

    pub fn add(&self, thread: Thread) {
        let mut running_queue = self.running_queue.lock();
        running_queue.push_back(thread);
    }

    pub fn pop(&self) -> Option<Thread> {
        let mut running_queue = self.running_queue.lock();
        // for t in running_queue.clone().into_iter() {
        //     println!("running queue: thread [{}]", t.tid());
        // }
        running_queue.pop_front()
    }

    pub fn blocked(&self, thread: Thread, timeout: Option<usize>) {
        let wakeup_time = timeout.map(|t| current_ms() + t);
        debug!(
            "Thread[{}] blocked, timeout: {:?} wakeup_time: {:?}",
            thread.tid(),
            timeout,
            wakeup_time
        );
        let mut blocked_queue = self.blocked_queue.lock();
        if let Some(wt) = wakeup_time {
            blocked_queue.insert(wt, thread);
        } else {
            blocked_queue.insert(usize::MAX, thread);
        }
    }

    pub fn get_wakeup_thread_by_time(&self, current_ms: usize) -> Option<Thread> {
        let mut blocked_queue = self.blocked_queue.lock();
        if let Some((nearest_wakeup_time, nearest_wakeup_thread)) = blocked_queue.first_key_value()
        {
            if *nearest_wakeup_time < current_ms {
                debug!(
                    "Thread[{}] is removed from blocked queue, wakeuptime: {} current time: {}",
                    nearest_wakeup_thread.tid(),
                    nearest_wakeup_time,
                    current_ms
                );
                return Some(blocked_queue.pop_first().unwrap().1);
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

    pub fn show_running_threads(&self) {
        let running_queue = self.running_queue.lock();
        for t in running_queue.iter() {
            debug!("Running Thread[{}]", t.tid());
        }
    }

    pub fn show_blocked_threads(&self) {
        let blocked_queue = self.blocked_queue.lock();
        for t in blocked_queue.iter() {
            debug!("Blocked Thread[{}], sleep time {}ms", t.1.tid(), t.0);
        }
    }
}

static SCHEDULER: Once<RoundRobinScheduler> = Once::new();

pub fn scheduler() -> &'static RoundRobinScheduler {
    if let Some(s) = SCHEDULER.get() {
        s
    } else {
        SCHEDULER.call_once(|| RoundRobinScheduler::new())
    }
}
