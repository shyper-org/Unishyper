#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use rust_shyper_os::arch::*;
use rust_shyper_os::exported::*;
use rust_shyper_os::*;


#[no_mangle]
fn main() {
    network_init();
    exit();
}
