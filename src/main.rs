#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
// #![feature(const_btree_new)]
#![feature(allocator_api)]

#[macro_use]
extern crate log;

mod arch;
mod driver;
mod logger;
mod lib;
mod panic;

#[no_mangle]
fn main(core_id: usize) {
    let _ = logger::init();
    info!("Hello, world! core {}", core_id);
}
