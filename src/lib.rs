#![no_std]
// Drop the #![no_main] attribute as it has no effect on library crates.
// #![no_main]
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
// use of unstable library feature 'step_trait': recently redesigned
// see issue #42168 <https://github.com/rust-lang/rust/issues/42168> for more information
// add `#![feature(step_trait)]` to the crate attributes to enable
#![feature(step_trait)]
// error[E0658]: use of unstable library feature 'core_intrinsics':
// intrinsics are unlikely to ever be stabilized,
// instead they should be used through stabilized interfaces in the rest of the standard library
#![feature(core_intrinsics)]

#[macro_use]
extern crate log;
// #[macro_use]
extern crate alloc;
#[macro_use]
extern crate derive_more;
#[macro_use]
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

pub mod libs;

pub use crate::libs::traits::ArchTrait;
pub use exported::*;
// This `irq_disable` is just for test, to be moved.
pub use crate::arch::irq::disable as irq_disable;

#[no_mangle]
pub extern "C" fn loader_main(core_id: usize) {
    arch::Arch::exception_init();

    if core_id == 0 {
        // Init serial output.
        crate::drivers::uart::init();
        mm::heap::init();
        let _ = logger::init();
        info!("heap init ok!!");
        mm::init();
    }

    #[cfg(feature = "smp")]
    board::launch_other_cores();

    board::init_per_core();
    info!("per core init ok on core [{}]", core_id);

    if core_id == 0 {
        board::init();
        info!("board init ok");
        logger::print_logo();
        // Init user main thread.
        extern "C" {
            fn main(arg: usize) -> !;
        }
        let t = crate::libs::thread::thread_alloc(None, main as usize, 123 as usize, 0, true);
        libs::thread::thread_wake(&t);
        // Init fs if configured.
        #[cfg(feature = "fs")]
        libs::fs::init();
        // Init shell if configured.
        #[cfg(feature = "terminal")]
        libs::terminal::init();
    }

    libs::cpu::cpu().schedule();

    extern "C" {
        fn pop_context_first(ctx: usize) -> !;
    }

    debug!("entering first thread...");
    match libs::cpu::cpu().running_thread() {
        None => panic!("no running thread"),
        Some(t) => {
            let sp = t.last_stack_pointer();
            unsafe { pop_context_first(sp) }
        }
    }
}
