use spin::Once;

use crate::board::BOARD_CORE_NUMBER;
use crate::libs::scheduler::scheduler;
use crate::libs::thread::Thread;
use crate::libs::traits::*;

pub struct Core {
    current_stack_pointer: usize,
    // pointer points at stack
    running_thread: Option<Thread>,
    idle_thread: Once<Thread>,
}

// Note: only the core itself can be allowed to access its `Core`
unsafe impl core::marker::Send for Core {}

unsafe impl core::marker::Sync for Core {}

const CORE: Core = Core {
    running_thread: None,
    current_stack_pointer: 0xDEAD_BEEF,
    idle_thread: Once::new(),
};

static mut CORES: [Core; BOARD_CORE_NUMBER] = [CORE; BOARD_CORE_NUMBER];
// static mut schedule_count: usize =  0;

impl Core {
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
                    idle_thread as usize,
                    core_id,
                    0,
                    true,
                );
                debug!(
                    "Alloc idle thread [{}] on core [{}]",
                    t.tid(),
                    crate::arch::Arch::core_id()
                );
                self.idle_thread.call_once(|| t).clone()
            }
            Some(t) => t.clone(),
        }
    }

    pub fn schedule(&mut self) {
        if let Some(t) = scheduler().pop() {
            self.run(t);
        } else {
            // debug!("scheduler empty, alloc idle thread\n");
            self.run(self.idle_thread());
            crate::arch::irq::enable();
        }
    }

    pub fn schedule_to(&mut self, t: Thread) {
        self.run(t);
    }

    fn run(&mut self, t: Thread) {
        use cortex_a::registers::TPIDRRO_EL0;
        use tock_registers::interfaces::Writeable;
        TPIDRRO_EL0.set(t.tid() as u64);

        if let Some(prev) = self.running_thread() {
            // Note: normal switch
            // debug!("switch thread from [{}] to [{}]", prev.tid(), t.tid());
            prev.set_last_stack_pointer(self.current_sp());

            // add back to scheduler queue
            if prev.runnable() {
                scheduler().add(prev.clone());
            }

            // debug!(
            //     "prev sp {:x}, next sp {:x}",
            //     self.current_sp(),
            //     t.stack_pointer()
            // );
        }
        self.set_running_thread(Some(t.clone()));
        self.set_current_sp(t.last_stack_pointer());
    }
}

#[inline(always)]
pub fn cpu() -> &'static mut Core {
    let core_id = crate::arch::Arch::core_id();
    unsafe { &mut CORES[core_id] }
}

#[no_mangle]
fn idle_thread(_arg: usize) {
    loop {
        crate::arch::Arch::wait_for_interrupt();
    }
}
