#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use rust_shyper_os::*;

#[macro_use]
extern crate alloc;

mod test;

#[allow(dead_code)]
extern "C" fn test_thread(arg: usize) {
    println!("test thread arg {}", arg);
    loop {
        thread_yield();
    }
}

#[no_mangle]
fn main() {
	println!("Hello world!");
    
    // thread_spawn(test_thread, 123);
    // println!("spawn thread, prepare to yield");
    // thread_yield();
    test::run_tests();
	// for i in 0..5 {
    //     println!("round [{}], yield to", i);
    //     thread_yield();
    //     println!("round [{}], yield back", i);
    // }
    // loop{}
    exit();
}
