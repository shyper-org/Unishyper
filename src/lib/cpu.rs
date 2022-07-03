use spin::Once;

use crate::arch::{ContextFrame, BOARD_CORE_NUMBER};
use crate::lib::scheduler::scheduler;
use crate::lib::thread::Thread;
use crate::lib::traits::*;

pub struct Core {
    context: Option<*mut ContextFrame>,
    // pointer points at stack
    running_thread: Option<Thread>,
    idle_thread: Once<Thread>,
}

// Note: only the core itself can be allowed to access its `Core`
unsafe impl core::marker::Send for Core {}

unsafe impl core::marker::Sync for Core {}

const CORE: Core = Core {
    context: None,
    running_thread: None,
    idle_thread: Once::new(),
};

static mut CORES: [Core; BOARD_CORE_NUMBER] = [CORE; BOARD_CORE_NUMBER];
// static mut schedule_count: usize =  0;

impl Core {
    // context

    pub fn context(&self) -> &ContextFrame {
        unsafe { self.context.unwrap().as_ref() }.unwrap()
    }

    pub fn context_mut(&self) -> &mut ContextFrame {
        unsafe { self.context.unwrap().as_mut() }.unwrap()
    }

    pub fn set_context(&mut self, ctx: *mut ContextFrame) {
        self.context = Some(ctx);
    }

    pub fn clear_context(&mut self) {
        self.context = None;
    }

    // thread
    pub fn running_thread(&self) -> Option<Thread> {
        self.running_thread.clone()
    }

    pub fn set_running_thread(&mut self, t: Option<Thread>) {
        self.running_thread = t
    }

    fn idle_thread(&self) -> Thread {
        match self.idle_thread.get() {
            None => {
                info!("Alloc idle thread on core {}:",crate::arch::Arch::core_id());
                let t = crate::lib::thread::thread_alloc(
                    idle_thread as usize,
                    crate::arch::Arch::core_id(),
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
            self.run(self.idle_thread());
        }
    }

    pub fn schedule_to(&mut self, t: Thread) {
        self.run(t);
    }

    fn run(&mut self, t: Thread) {
        if let Some(prev) = self.running_thread() {
            trace!("switch thread from [{}] to [{}]", prev.tid(), t.tid());
            // Note: normal switch
            prev.set_context(*self.context());
            // add back to scheduler queue
            if prev.runnable() {
                scheduler().add(prev.clone());
            }
            trace!("next ctx:\n {}", t.context());
            *self.context_mut() = t.context();
        } else {
            trace!("run thread {}",t.tid());
            if self.context.is_some() {
                // Note: previous process has been destroyed
                // debug!("previous process has been destroyed, next ctx:\n {}", t.context());
                *self.context_mut() = t.context();
            } else {
                // Note: this is first run
                // `loader_main` prepare the context to stack
                trace!("first run thread {}", t.tid());
            }
        }
        self.set_running_thread(Some(t.clone()));
    }
}

pub fn cpu() -> &'static mut Core {
    let core_id = crate::arch::Arch::core_id();
    unsafe { &mut CORES[core_id] }
}

#[no_mangle]
fn idle_thread(_arg: usize) {
    loop {
        trace!("idle_thread {}\n", _arg);
        // loop{}
        crate::arch::Arch::wait_for_interrupt();
    }
}
