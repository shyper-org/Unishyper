use spin::Once;

use crate::board::BOARD_CORE_NUMBER;
use crate::libs::thread::Thread;
use crate::libs::traits::*;
use crate::libs::scheduler::{Scheduler, ScheduerType};

pub type CoreId = usize;

pub struct Core {
    // Stack pointer of user mode.
    running_thread: Option<Thread>,
    current_stack_pointer: usize,
    idle_thread: Once<Thread>,
    sched: ScheduerType,
    #[cfg(target_arch = "x86_64")]
    arch_specific_data: crate::arch::Cpu,
}

// Note: only the core itself can be allowed to access its `Core`
unsafe impl core::marker::Send for Core {}

unsafe impl core::marker::Sync for Core {}

const CORE: Core = Core {
    running_thread: None,
    current_stack_pointer: 0xDEAD_BEEF,
    idle_thread: Once::new(),
    sched: ScheduerType::None,
    #[cfg(target_arch = "x86_64")]
    arch_specific_data: crate::arch::Cpu::new(),
};

static mut CORES: [Core; BOARD_CORE_NUMBER] = [CORE; BOARD_CORE_NUMBER];

impl Core {
    #[cfg(target_arch = "x86_64")]
    pub fn get_cpu_data(&'static mut self) -> &'static mut crate::arch::Cpu {
        &mut self.arch_specific_data
    }

    pub fn set_current_sp(&mut self, sp: usize) {
        self.current_stack_pointer = sp
    }

    pub fn current_sp(&self) -> usize {
        self.current_stack_pointer
    }

    // thread
    pub fn running_thread(&self) -> Option<Thread> {
        self.running_thread.clone()
    }

    fn running_thread_ref(&self) -> Option<&Thread> {
        self.running_thread.as_ref()
    }

    pub fn set_running_thread(&mut self, t: Option<Thread>) {
        self.running_thread = t
    }

    /// Alloc idle thread on each core when there is no running thread on scheduler.
    /// Each idle only inits once.
    ///
    /// Note: idle thread id depends on core number,
    /// for example, core 0's idle thread id is 11, core 1's idle thread id is 22.
    fn idle_thread(&self) -> Thread {
        match self.idle_thread.get() {
            None => {
                let core_id = crate::arch::Arch::core_id();
                let idle_thread_id = (core_id + 1) * 10 + (core_id + 1);
                let t = crate::libs::thread::thread_alloc(
                    Some(idle_thread_id),
                    Some(core_id),
                    idle_thread as usize,
                    core_id,
                    0,
                    true,
                );
                debug!(
                    "Alloc idle thread [{}] on core [{}], context on sp {:x}",
                    t.id(),
                    crate::arch::Arch::core_id(),
                    t.last_stack_pointer()
                );
                self.idle_thread.call_once(|| t).clone()
            }
            Some(t) => t.clone(),
        }
    }

    pub fn set_scheduler(&mut self, scheduler: ScheduerType) {
        self.sched = scheduler;
        let core_id = crate::arch::Arch::core_id();
        info!("Scheduler init ok on core [{}]", core_id);
    }

    pub fn scheduler(&self) -> &impl Scheduler {
        match &self.sched {
            ScheduerType::None => panic!("scheduler is None"),
            ScheduerType::PerCoreSchedRoundRobin(rr) => rr,
            ScheduerType::GlobalSchedRoundRobin => crate::libs::scheduler::global_scheduler(),
        }
    }

    pub fn get_next_thread(&mut self) -> Thread {
        self.scheduler().pop().unwrap_or_else(|| self.idle_thread())
    }

    pub fn schedule(&mut self) {
        // Get prev thread.
        let prev = self.running_thread_ref().unwrap_or_else(|| {
            panic!(
                "No running thread on core [{}], something is wrong!!!",
                crate::arch::Arch::core_id()
            )
        });

        // Add prev thread back to scheduler queue.
        if prev.runnable() {
            self.scheduler().add(prev.clone());
        }

        // Get next thread from scheduler.
        let next = self.scheduler().pop().unwrap_or_else(|| {
            if prev.runnable() {
                prev.clone()
            } else {
                self.idle_thread()
            }
        });

        // debug!("cpu schedule\nprev {} to next {}", prev.id(), next.id());

        if prev.eq(&next) {
            return;
        }

        unsafe {
            let prev_ctx_ptr = prev.ctx_mut_ptr();
            let next_ctx_ptr = next.ctx_mut_ptr();
            // assert!(Arc::strong_count(&prev) > 1);
            // assert!(Arc::strong_count(&next) >= 1);
            crate::arch::Arch::set_thread_id(next.id().as_u64());
            crate::arch::Arch::set_tls_ptr(next.get_tls_ptr() as u64);
            let next_is_not_run = next.in_trap_context();
            if next_is_not_run {
                next.set_in_yield_context();
            }
            let next_stack_pointer = next.last_stack_pointer();
            self.set_running_thread(Some(next));

            if next_is_not_run {
                // debug!("switch_to_trap_ctx on {:#x}", next_stack_pointer);
                (*prev_ctx_ptr).switch_to_trap_ctx(next_stack_pointer);
            } else {
                // debug!("switch_to_yield_ctx on {:#p}", next_ctx_ptr);
                (*prev_ctx_ptr).switch_to_yield_ctx(&*next_ctx_ptr);
            }
        }
    }
}

/// Get current CPU structure.
#[inline(always)]
pub fn cpu() -> &'static mut Core {
    let core_id = crate::arch::Arch::core_id();
    unsafe { &mut CORES[core_id] }
}

/// Get target CPU structure of given cpu id.
#[inline(always)]
pub fn get_cpu(core_id: usize) -> &'static mut Core {
    unsafe { &mut CORES[core_id] }
}

#[no_mangle]
fn idle_thread(_arg: usize) {
    loop {
        crate::arch::Arch::wait_for_interrupt();
    }
}
