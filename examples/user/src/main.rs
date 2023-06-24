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

// #[allow(dead_code)]
// extern "C" fn test_thread(arg: usize) {
//     println!("enter test_thread {}", arg);
//     // irq_disable();
//     for i in 0..5 {
//         println!("test_thread round [{}], yield to", i);
//         thread_yield();
//         println!("test_thread round [{}], yield back", i);
//         // i+=1;
//     }
//     loop{}
// }

#[no_mangle]
fn main() {
    println!("Hello world!");

    // thread_spawn(test_thread, 123);
    // println!("spawn thread, prepare to yield");
    // thread_yield();
    // test::run_tests();
    // for i in 0..5 {
    //     println!("round [{}], yield to", i);
    //     thread_yield();
    //     println!("round [{}], yield back", i);
    // }
    // println!("enter loop on main!!!");
    // loop{}
    exit();
}
