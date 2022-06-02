#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
#![feature(const_btree_new)]
#![feature(allocator_api)]
#![feature(never_type)]

#[macro_use]
extern crate log;
extern crate alloc;
extern crate static_assertions;

pub mod arch;
pub mod driver;
pub mod lib;
pub mod logger;
pub mod mm;
pub mod panic;
pub mod util;
pub mod board;
pub mod exported;

pub use crate::lib::traits::ArchTrait;

#[no_mangle]
fn loader_main(core_id: usize) {
    crate::arch::Arch::exception_init();

    println!("enter main, core {}", core_id);
    mm::heap::init();
    let _ = logger::init();
    info!("heap init ok!!");
    mm::page_pool::init();
    info!("page pool init ok");
    board::init();
    info!("board init ok");

    let stack_frame = crate::mm::page_pool::page_alloc().expect("fail to allocate test thread stack");

    println!(
        "thread user main, stack frame pa: 0x{:x} kva: 0x{:x}",
        stack_frame.pa(),
        stack_frame.kva()
    );

    extern "C" {
        fn main(arg: usize) -> !;
    }

    let t = crate::lib::thread::new_kernel(
        main as usize,
        stack_frame.kva() + crate::arch::PAGE_SIZE,
        123 as usize,
    );
    lib::thread::thread_wake(&t);

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
