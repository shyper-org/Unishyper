#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

#![feature(allocator_api)]

use unishyper::*;

#[macro_use]
extern crate alloc;

// mod test;

#[no_mangle]
fn main() {
    println!("Hello world!");
    // test::run_tests();
}
