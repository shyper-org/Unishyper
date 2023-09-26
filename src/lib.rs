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
// #![cfg_attr(feature = "std", feature(atomic_mut_ptr))]
// error[E0658]: use of unstable library feature 'strict_provenance'
// note: see issue #95228 <https://github.com/rust-lang/rust/issues/95228> for more information
// help: add `#![feature(strict_provenance)]` to the crate attributes to enable
#![cfg_attr(feature = "std", feature(strict_provenance))]
// error[E0658]: use of unstable library feature 'is_some_and'
// note: see issue #93050 <https://github.com/rust-lang/rust/issues/93050> for more information
// help: add `#![feature(is_some_and)]` to the crate attributes to enable
// #![cfg_attr(feature = "std", feature(is_some_and))]
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
// error[E0658]: use of unstable library feature 'variant_count'
// note: see issue #73662 <https://github.com/rust-lang/rust/issues/73662> for more information
#![feature(variant_count)]
// error[E0658]: use of unstable library feature 'ip_in_core'
// see issue #108443 <https://github.com/rust-lang/rust/issues/108443> for more information
#![feature(ip_in_core)]
#![feature(stmt_expr_attributes)]
#![feature(associated_type_defaults)]
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
// mod drivers;

#[cfg_attr(feature = "axdriver", path = "drivers/axmod.rs")]
#[cfg_attr(not(feature = "axdriver"), path = "drivers/mod.rs")]
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
pub use arch::page_table;
pub use panic::random_panic;

// pub static mut START_CYCLE: u64 = 0;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn print_built_info() {
    println!(
        "Unishyper for [{platform}] on [{arch}].\nBuilt at [{build_time}] by {hostname} of {rustc_version}",
        platform = env!("MACHINE"),
        arch = env!("ARCH"),
        build_time = env!("BUILD_TIME"),
        hostname = env!("HOSTNAME"),
        rustc_version = built_info::RUSTC_VERSION,
    );
    println!("Enabled features:");
    print!("\t[");
    for f in built_info::FEATURES_LOWERCASE {
        print!(" \"{f}\",");
    }
    println!("]");
}

#[no_mangle]
pub extern "C" fn loader_main(core_id: usize) {
    // Init output.
    //let saki=page_table::page_table().lock();
    if core_id == 0 {
        // Init serial output.
        #[cfg(feature = "serial")]
        drivers::uart::init();
        logger::init();
        print_built_info();
    }

    arch::Arch::exception_init();

    if core_id == 0 {
        libs::timer::init();
        mm::heap::init();
        mm::allocator_init();
        // After Page allocator and Frame allocator init finished, init user page table.
        arch::Arch::page_table_init();
        // debug!("page table init ok");

        #[cfg(feature = "smp")]
        board::launch_other_cores();
    }
    
    // if core_id!=0 {
    //     //println!("ok!,i'm in");
    //     let mut i=1;
    //     loop{
            
    //         i=i+1;
    //         if i>1000000000 {
    //             //println!("ok!,i'm out");
    //             break;
    //         }
    //     }
        
    // }
    board::init_per_core();
    println!("core_id iis {}",core_id);
    info!("per core init ok on core [{}]", core_id);
    
    // // Init schedule for per core.
    libs::scheduler::init();
    
    if core_id == 0 {
        board::init();
        
        #[cfg(feature = "net")]
        libs::net::init();
        #[cfg(feature = "fs")]
        libs::fs::init();
        
        // #[cfg(feature = "zone")]
        zone::zone_init();
        
        info!("board init ok");
        extern "Rust" {
            fn main(arg: usize) -> !;
        }
        let start = if cfg!(feature = "std") {
            extern "C" {
                fn runtime_entry(argc: i32, argv: *const *const u8, env: *const *const u8) -> !;
            }
            runtime_entry as usize
        } else {
            fn main_wrapper(main: extern "C" fn(usize), arg: usize) -> ! {
                main(arg);
                exit()
            }
            main_wrapper as usize
        };
        // Init user first thread on core 0 by default.
        let t = libs::thread::thread_alloc(None, Some(core_id), start, main as usize, 123, true);
        libs::thread::thread_wake(&t);
        t.set_in_yield_context();
        arch::Arch::set_thread_id(t.id().as_u64());
        arch::Arch::set_tls_ptr(t.get_tls_ptr() as u64);
        libs::cpu::cpu().set_running_thread(Some(t));
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
