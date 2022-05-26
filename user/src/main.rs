#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

use rust_shyper_os::*;

#[no_mangle]
fn test_thread(_arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
        "test_thread, core {} _arg {} curentel {}",
        core_id,
        _arg,
        crate::arch::Arch::curent_privilege()
    );
    loop{}
    // crate::arch::Arch::wait_for_interrupt();
}

#[no_mangle]
fn main() {
    println!("Hello, world!");

    for i in 0..10 {
        let stack_frame =
            crate::mm::page_pool::page_alloc().expect("fail to allocate test thread stack");
        let t = crate::lib::thread::new_kernel(
            test_thread as usize,
            stack_frame.kva() + crate::arch::PAGE_SIZE,
            i as usize,
        );
        println!(
            "thread[{}] stack frame {:x} kva{:x}",
            i,
            stack_frame.pa(),
            stack_frame.kva()
        );
        crate::lib::thread::thread_wake(&t);
    }
    loop{}
}
