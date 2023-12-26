#![no_std]
// error: requires `start` lang_item
#![no_main]
// error[E0658]: use of unstable library feature 'format_args_nl': `format_args_nl` is only for internal language use and is subject to change
// help: add `#![feature(format_args_nl)]` to the crate attributes to enable
// note: this error originates in the macro `println` (in Nightly builds, run with -Z macro-backtrace for more info)
#![feature(format_args_nl)]

extern crate alloc;

// use alloc::boxed::Box;

use unishyper::*;

mod resource;
// mod sem;
// mod fs;

#[no_mangle]
fn main() {
    println!("Hello, world!");
    thread_spawn(resource::test_recover, 123);
    // thread_spawn(sem::semaphore_test, 123);
    // thread_spawn(fs::test_fs, 123);
}
