use crate::mm::page_pool::*;
use crate::lib::cpu::cpu;
use crate::lib::thread::Thread;


pub type Error = usize;

fn current_thread() -> core::result::Result<Thread, Error> {
    match cpu().running_thread() {
      None => Err(ERROR_INTERNAL),
      Some(t) => Ok(t),
    }
}

pub fn exit() -> ! {
    let result = current_thread();
    match result {
        Ok(t) => {
            crate::lib::thread::thread_destroy(t);
        }
        Err(_) => {
            panic!("failed to get current_thread");
        }
    }
    loop{}
}