use alloc::boxed::Box;

use crate::lib::thread::{current_thread, new_kernel, Tid};

// Todo: may use fuction closure as parameters.
pub fn thread_spawn(func: extern "C" fn(usize), _arg: usize) -> Tid {
    // Alloc stack space.
    let stack_frame =
        crate::mm::page_pool::page_alloc().expect("fail to allocate test thread stack");

    info!(
        "thread user main, stack frame pa: 0x{:x} kva: 0x{:x}",
        stack_frame.pa(),
        stack_frame.kva()
    );

    let main = move |arg| {
        func(arg);
    };

    let p = Box::into_raw(Box::new(main));

    extern "C" fn thread_start(main: usize) -> usize {
        unsafe {
            Box::from_raw(main as *mut Box<dyn FnOnce()>)();
        }
        exit();
        0
    }
    let _t = match current_thread() {
        Ok(t) => t,
        Err(_) => {
            panic!("no current thread!");
        }
    };
    let child_thread = new_kernel(
        thread_start as usize,
        stack_frame.kva() + crate::arch::PAGE_SIZE,
        p as *mut _ as usize,
    );
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
