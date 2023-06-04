mod sched_rr;

pub enum ScheduerType {
    /// No SMP support, no scheduler.
    None,
    /// Scheduling on multiple cores with each queue in each processor.
    PerCoreSchedRoundRobin(sched_rr::RoundRobinScheduler),
    /// Scheduling on multiple cores with a global queue.
    GlobalSchedRoundRobin,
}

pub trait Scheduler {
    fn add_front(&self, thread: crate::libs::thread::Thread);
    fn add(&self, thread: crate::libs::thread::Thread);
    fn pop(&self) -> Option<crate::libs::thread::Thread>;
    fn blocked(&self, thread: crate::libs::thread::Thread, timeout: Option<usize>);
    fn get_wakeup_thread_by_time(&self, current_ms: usize) -> Option<crate::libs::thread::Thread>;
}

pub fn init() {
    if cfg!(feature = "scheduler-percore") {
        let core_scheduler =
            ScheduerType::PerCoreSchedRoundRobin(sched_rr::RoundRobinScheduler::new());
        crate::libs::cpu::cpu().set_scheduler(core_scheduler);
        info!("Per core scheduler init ok");
    } else {
        info!("Init global scheduler...");
        crate::libs::cpu::cpu().set_scheduler(ScheduerType::GlobalSchedRoundRobin);
        info!("Global scheduler init ok");
    }
}

use spin::Once;

static GLOBAL_SCHEDULER: Once<sched_rr::RoundRobinScheduler> = Once::new();

pub fn global_scheduler() -> &'static sched_rr::RoundRobinScheduler {
    if let Some(s) = GLOBAL_SCHEDULER.get() {
        s
    } else {
        GLOBAL_SCHEDULER.call_once(|| sched_rr::RoundRobinScheduler::new())
    }
}

// static SCHEDULER: ScheduerType = ScheduerType::None;

// pub fn scheduler() -> &mut impl Scheduler {
//     match &mut self.sched {
//         SchedType::None => panic!("scheduler is None"),
//         SchedType::SchedRR(rr) => rr,
//     }
//     if let Some(s) = SCHEDULER.get() {
//         s
//     } else {
//         SCHEDULER.call_once(|| RoundRobinScheduler::new())
//     }
// }
