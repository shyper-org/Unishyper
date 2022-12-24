#![no_std]
// Drop the #![no_main] attribute as it has no effect on library crates.
// #![no_main]
#![feature(alloc_error_handler)]
// #![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
// warning: the feature `const_btree_new` has been stable since 1.66.0 and no longer requires an attribute to enable
// #![feature(const_btree_new)]
#![feature(allocator_api)]
#![feature(never_type)]
#![feature(asm_const)]
#![feature(drain_filter)]
// warning: the feature `map_first_last` has been stable since 1.66.0 and no longer requires an attribute to enable
// #![feature(map_first_last)]
// use of unstable library feature 'step_trait': recently redesigned
// see issue #42168 <https://github.com/rust-lang/rust/issues/42168> for more information
// add `#![feature(step_trait)]` to the crate attributes to enable
#![feature(step_trait)]
// error[E0658]: use of unstable library feature 'core_intrinsics':
// intrinsics are unlikely to ever be stabilized,
// instead they should be used through stabilized interfaces in the rest of the standard library
#![feature(core_intrinsics)]
// error[E0658]: use of unstable library feature 'new_uninit'
// note: see issue #63291 <https://github.com/rust-lang/rust/issues/63291> for more information
// help: add `#![feature(new_uninit)]` to the crate attributes to enable
#![feature(new_uninit)]
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

pub use libs::traits::ArchTrait;
pub use exported::*;
// This `irq_disable` is just for test, to be moved.
pub use arch::irq::disable as irq_disable;

#[no_mangle]
pub extern "C" fn loader_main(core_id: usize) {
    arch::Arch::exception_init();

    if core_id == 0 {
        // Init serial output.
        #[cfg(feature = "serial")]
        drivers::uart::init();
        logger::print_logo();
        mm::heap::init();
        let _ = logger::init();
        info!("heap init ok!!");

        // use core::cell::Cell;
        // use core::cell::RefCell;
        // use alloc::vec;
        // use alloc::rc::Rc;
        // use alloc::boxed::Box;
        // use spin::Mutex;
        // let test = Cell::new(0 as u8);
        // test.replace(val);

        // println!(
        //     "sizeof Mutex<*mut u8> {}",
        //     core::mem::size_of::<Mutex<*mut u8>>()
        // );
        // println!(
        //     "sizeof Cell<*mut u8> {}",
        //     core::mem::size_of::<Cell<*mut u8>>()
        // );
        // println!(
        //     "sizeof Rc<Cell<*mut u8>> {}",
        //     core::mem::size_of::<Rc<Cell<*mut u8>>>()
        // );
        // println!(
        //     "sizeof RefCell<*mut u8> {}",
        //     core::mem::size_of::<RefCell<*mut u8>>()
        // );
        // println!("sizeof *mut u8 {}", core::mem::size_of::<*mut u8>());
        // // println!("sizeof mut u8 {}", core::mem::size_of::<mut u8>());
        // println!("sizeof u8 {} end", core::mem::size_of::<u8>());
        // let vec1 = Box::<[*mut u8]>::new_zeroed_slice(128);
        // println!("end");
        // let vec1 = unsafe { vec1.assume_init() };
        // println!("end");
        // println!("sizeof Box<[u8]> {}", vec1.len());
        // println!("end");

        // loop {}

        mm::allocator_init();
        // After Page allocator and Frame allocator init finished, init user page table.
        arch::Arch::page_table_init();
        info!("page table init ok");

        #[cfg(feature = "smp")]
        board::launch_other_cores();
    }

    board::init_per_core();
    info!("per core init ok on core [{}]", core_id);

    // Init schedule for per core.
    libs::scheduler::init();

    if core_id == 0 {
        board::init();
        info!("board init ok");
        // logger::print_logo();
        // Init user main thread on core 0 by default.
        extern "C" {
            #[allow(unused)]
            fn main(arg: usize) -> !;
            fn runtime_entry(argc: i32, argv: *const *const u8, env: *const *const u8) -> !;
        }
        let t =
            libs::thread::thread_alloc(None, Some(core_id), runtime_entry as usize, 123 as usize, 0, true);
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

    let sp = match libs::cpu::cpu().running_thread() {
        None => panic!("no running thread"),
        Some(t) => {
            t.last_stack_pointer()
        }
    };
    debug!("entering first thread on sp {:x}...", sp);
    unsafe { pop_context_first(sp) }
}
