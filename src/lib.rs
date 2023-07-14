#![cfg_attr(not(feature = "std"), no_std)]
// Drop the #![no_main] attribute as it has no effect on library crates.
// #![no_main]
#![feature(alloc_error_handler)]
#![cfg_attr(not(feature = "std"), feature(panic_info_message))]
#![feature(format_args_nl)]
#![feature(lang_items)]
// warning: the feature `const_btree_new` has been stable since 1.66.0 and no longer requires an attribute to enable
// warning: the feature `const_btree_new` has been partially stabilized since 1.66.0 and is succeeded by the feature `const_btree_len`
#![cfg_attr(not(feature = "std"), feature(const_btree_len))]
#![feature(allocator_api)]
#![feature(never_type)]
#![feature(asm_const)]
// #![feature(drain_filter)]
// warning: the feature `map_first_last` has been stable since 1.66.0 and no longer requires an attribute to enable
// #![cfg_attr(not(feature = "std"), feature(map_first_last))]
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
#![cfg_attr(feature = "std", feature(new_uninit))]
// error[E0658]: use of unstable library feature 'atomic_mut_ptr': recently added
// note: see issue #66893 <https://github.com/rust-lang/rust/issues/66893> for more information
// help: add `#![feature(atomic_mut_ptr)]` to the crate attributes to enable
#![cfg_attr(feature = "std", feature(atomic_mut_ptr))]
// error[E0658]: use of unstable library feature 'strict_provenance'
// note: see issue #95228 <https://github.com/rust-lang/rust/issues/95228> for more information
// help: add `#![feature(strict_provenance)]` to the crate attributes to enable
#![cfg_attr(feature = "std", feature(strict_provenance))]
// error[E0658]: use of unstable library feature 'is_some_and'
// note: see issue #93050 <https://github.com/rust-lang/rust/issues/93050> for more information
// help: add `#![feature(is_some_and)]` to the crate attributes to enable
#![cfg_attr(feature = "std", feature(is_some_and))]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
// error: `MaybeUninit::<T>::zeroed` is not yet stable as a const fn
#![feature(const_maybe_uninit_zeroed)]
// #![feature(asm_sym)]
#![feature(naked_functions)]
// note: see issue #76001 <https://github.com/rust-lang/rust/issues/76001> for more information
#![feature(inline_const)]
// note: see issue #71941 <https://github.com/rust-lang/rust/issues/71941> for more information
// help: add `#![feature(nonnull_slice_from_raw_parts)]` to the crate attributes to enable
// #![feature(nonnull_slice_from_raw_parts)]
#![feature(alloc_layout_extra)]
#![feature(slice_ptr_get)]

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
pub use mm::heap::Global;

pub use panic::random_panic;

#[no_mangle]
pub extern "C" fn loader_main(core_id: usize) {
    arch::Arch::exception_init();
    if core_id == 0 {
        // Init serial output.
        #[cfg(feature = "serial")]
        drivers::uart::init();
        logger::init();
        libs::timer::init();
        mm::heap::init();
        mm::allocator_init();
        // After Page allocator and Frame allocator init finished, init user page table.
        arch::Arch::page_table_init();
        // debug!("page table init ok");

        #[cfg(feature = "smp")]
        board::launch_other_cores();
    }

    board::init_per_core();
    info!("per core init ok on core [{}]", core_id);

    // // Init schedule for per core.
    libs::scheduler::init();

    if core_id == 0 {
        board::init();
        info!("board init ok");
        // Init user first thread on core 0 by default.
        extern "C" {
            #[cfg(not(feature = "std"))]
            fn main(arg: usize) -> !;
            #[cfg(feature = "std")]
            fn runtime_entry(argc: i32, argv: *const *const u8, env: *const *const u8) -> !;
        }
        #[cfg(not(feature = "std"))]
        let start = main as usize;
        #[cfg(feature = "std")]
        let start = runtime_entry as usize;
        let t = libs::thread::thread_alloc(None, Some(core_id), start, 123 as usize, 0, true);
        libs::thread::thread_wake(&t);
        t.set_in_yield_context();
        arch::Arch::set_thread_id(t.tid() as u64);
        arch::Arch::set_tls_ptr(t.get_tls_ptr() as u64);
        libs::cpu::cpu().set_running_thread(Some(t));
        // Init fs if configured.
        #[cfg(feature = "fs")]
        libs::fs::init();
        // Init shell if configured.
        #[cfg(feature = "terminal")]
        libs::terminal::init();
    }

    // Enter first thread.
    // On core 0, this should be user's main thread.
    // On other cores, this may be idle thread.
    let t = libs::cpu::cpu().get_next_thread();

    let sp = t.last_stack_pointer();
    debug!("entering first thread on sp {:#x}...", sp);
    crate::arch::Arch::pop_context_first(sp)
}
