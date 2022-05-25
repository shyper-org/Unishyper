#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
#![feature(const_btree_new)]
#![feature(allocator_api)]

#[macro_use]
extern crate log;
extern crate alloc;
extern crate static_assertions;

mod arch;
mod driver;
mod lib;
mod logger;
mod mm;
mod panic;
mod util;
mod board;

use crate::lib::traits::ArchTrait;

#[no_mangle]
fn test_thread(_arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    loop {
        info!("test_thread, core {} _arg {} curentel {}", core_id, _arg, crate::arch::Arch::curent_privilege());
        // crate::arch::Arch::wait_for_interrupt();
    }
}

#[no_mangle]
fn main(core_id: usize) {
    crate::arch::Arch::exception_init();

    println!("enter main, core {}", core_id);
    mm::heap::init();
    let _ = logger::init();
    info!("heap init ok!!");
    mm::page_pool::init();
    info!("page pool init ok");
    board::init();
    info!("board init ok");

    for i in 0..10 {
        let stack_frame = crate::mm::page_pool::page_alloc().expect("fail to allocate test thread stack");
        let t = crate::lib::thread::new_kernel(
            test_thread as usize,
            stack_frame.kva() + crate::arch::PAGE_SIZE,
            i + 1 as usize,
        );
        info!("thread[{}] stack frame {:x} kva{:x}", i, stack_frame.pa(), stack_frame.kva());
        lib::thread::thread_wake(&t);
    }

    lib::cpu::cpu().schedule();

    extern "C" {
        fn pop_context_first(ctx: usize, core_id: usize) -> !;
    }
    match lib::cpu::cpu().running_thread() {
        None => panic!("no running thread"),
        Some(t) => {
            let ctx = t.context();
            unsafe {
                pop_context_first(&ctx as *const _ as usize, core_id);
            }
        }
    }
}
