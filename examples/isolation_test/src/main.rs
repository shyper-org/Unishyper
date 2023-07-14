#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use unishyper::*;

extern "C" fn test_thread_write(test_var_addr: usize) {
    unsafe {
        let test_var = test_var_addr as *mut usize;

        println!(
            "On test_thread, try to write test var on {:#x}",
            test_var_addr,
        );

        *test_var = 321;

        println!(
            "On test_thread, test var changed to {:?}",
            test_var.as_mut()
        );
    }

    thread_yield();
    loop {}
}

extern "C" fn test_thread_read(test_var_addr: usize) {
    let test;
    unsafe {
        let test_var = test_var_addr as *mut usize;

        println!(
            "On test_thread, try to read test var on {:#x}",
            test_var_addr
        );

        test = *test_var;
    }
    println!("On test_thread, test var is {:?}", test);

    thread_yield();
    loop {}
}

#[no_mangle]
fn main() {
    println!("Hello, world!");

    let test_var = 123 as usize;

    println!("On main thread, test var is {}", test_var);

    thread_spawn(test_thread_write, &test_var as *const _ as usize);
    // thread_spawn(test_thread_read, &test_var as *const _ as usize);

    loop {
        thread_yield();
        println!("Back to main thread, test var is {}", test_var);
        exit();
    }
}
