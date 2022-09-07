#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
#![feature(const_btree_new)]
#![feature(allocator_api)]
#![feature(never_type)]
#![feature(asm_const)]
#![feature(drain_filter)]
#![feature(map_first_last)]
#![feature(const_fn_trait_bound)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;
extern crate static_assertions;

#[macro_use]
mod macros;

mod arch;
mod board;
mod drivers;
mod exported;
mod logger;
mod mm;
mod panic;
mod util;
pub mod lib;

pub use crate::lib::traits::ArchTrait;
pub use exported::*;

use crate::util::irqsave;

#[no_mangle]
fn loader_main(core_id: usize) {
    crate::arch::Arch::exception_init();

    println!("enter main, core {}", core_id);
    if core_id == 0 {
        mm::heap::init();
        let _ = logger::init();
        info!("heap init ok!!");
        mm::page_pool::init();
        info!("page pool init ok");
    }

    #[cfg(feature = "smp")]
    board::launch_other_cores();
    
    board::init_per_core();
    info!("per core init ok on core [{}]", core_id);

    if core_id == 0 {
        irqsave(|| {
            board::init();
            info!("board init ok");

            extern "C" {
                fn main(arg: usize) -> !;
            }

            let t = crate::lib::thread::thread_alloc(main as usize, 123 as usize);
            lib::thread::thread_wake(&t);

            println!(concat!(
                "\nHello world!\n\n",
                "Welcome to shyper lightweight os...\n\n",
                "====== entering first thread ======>>>\n"
            ));
        });
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
