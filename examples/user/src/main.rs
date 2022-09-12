#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use rust_shyper_os::*;

#[macro_use]
extern crate alloc;

mod test;

#[no_mangle]
fn main() {

	println!("Hello world!");
    
    test::run_tests();
	
    exit();
}
