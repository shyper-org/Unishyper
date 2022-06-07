use alloc::boxed::Box;

use crate::lib::thread::{
    Tid,
    current_thread, 
    thread_alloc,
    thread_wake,
};

// Todo: may use fuction closure as parameters.
pub fn thread_spawn(func: extern "C" fn(usize), _arg: usize) -> Tid {
    let main = move |arg| {
        info!("main start");
        func(arg);
        info!("main end");
    };

    let p = Box::into_raw(Box::new(main));

    extern "C" fn thread_start(main: usize) -> usize {
        info!("thread_start main: {:x}", main);
        unsafe {
            Box::from_raw(main as *mut Box<dyn FnOnce()>)();
        }
        info!("thread_exit");
        exit();
        0
    }
    let _t = match current_thread() {
        Ok(t) => t,
        Err(_) => {
            panic!("no current thread!");
        }
    };
    let child_thread = thread_alloc(
        thread_start as usize,
        p as *mut _ as usize,
    );
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
