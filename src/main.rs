#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
#![feature(const_btree_new)]
#![feature(allocator_api)]

#[macro_use]
extern crate log;
extern crate alloc;

mod arch;
mod driver;
mod lib;
mod logger;
mod mm;
mod panic;
mod util;

#[no_mangle]
fn main(core_id: usize) {
    println!("enter main, core {}", core_id);
    mm::heap::init();
    let _ = logger::init();
    info!("heap init ok!!");
    mm::page_pool::init();
    info!("page pool init ok");

    lib::cpu::cpu().schedule();
}
