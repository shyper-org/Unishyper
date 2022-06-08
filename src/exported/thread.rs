use crate::lib::thread::{current_thread, thread_alloc2, thread_wake, Tid};

// Todo: may use fuction closure as parameters.
pub fn thread_spawn(func: extern "C" fn(usize), arg: usize) -> Tid {
    debug!("thread_spawn func: {:x} arg: {}", func as usize, arg);

    extern "C" fn thread_start(func: extern "C" fn(usize), arg: usize) -> usize {
        func(arg);
        exit();
        0
    }

    let _t = match current_thread() {
        Ok(t) => t,
        Err(_) => {
            panic!("no current thread!");
        }
    };
    let child_thread = thread_alloc2(thread_start as usize, func as usize, arg);
    thread_wake(&child_thread);
    child_thread.tid()
}

pub fn exit() {
    let result = current_thread();
    match result {
        Ok(t) => {
            crate::lib::thread::thread_destroy(t);
        }
        Err(_) => {
            panic!("failed to get current_thread");
        }
    }
    loop {}
}
