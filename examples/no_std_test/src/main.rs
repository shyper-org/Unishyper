#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]
#![feature(allocator_api)]

use unishyper::*;

#[macro_use]
extern crate alloc;

mod test;

// use unishyper::START_CYCLE;

#[no_mangle]
fn main() {
    // unsafe {
    //     let start_cycle = START_CYCLE;
    //     let current_cycle = current_cycle() as u64;
    //     println!("\n start cycle {start_cycle}");
    //     println!("\n current_cycle {current_cycle}");
    //     println!("\n cycles  {}", current_cycle - start_cycle);
    // }
    println!("Hello world!");
    test::run_tests();
}
