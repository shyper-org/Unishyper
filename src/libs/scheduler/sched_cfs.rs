use crate::libs::thread::Thread;

use super::Scheduler;

pub struct CFSScheduler {}

impl CFSScheduler {
    pub fn new() -> Self {
        CFSScheduler {}
    }
}

impl Scheduler for CFSScheduler {
    fn add_front(&self, _thread: Thread) {
        todo!()
    }

    fn add(&self, _thread: Thread) {
        todo!()
    }

    fn pop(&self) -> Option<Thread> {
        todo!()
    }

    fn blocked(&self, _thread: Thread, _timeout: Option<usize>) {
        todo!()
    }

    fn get_wakeup_thread_by_time(&self, _current_ms: usize) -> Option<Thread> {
        todo!()
    }
}
