#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

extern crate alloc;

extern crate ring;
extern crate data_encoding;

use ring::digest::{Context, Digest, SHA256};
use data_encoding::HEXUPPER;

use alloc::vec::Vec;

use unishyper::*;
use unishyper::shyperstd as std;

use std::io::{BufReader, Read, Write};

// protected_global_var!(static mut TEST_PROTECTED_GLOCAL: usize = 123);

#[zone_protected::global_var]
static mut TEST_PROTECTED_GLOCAL: usize = 123;

static mut TEST_SHARED_GLOCAL: usize = 456;

#[allow(unused)]
fn test_stack_var_rw() {
    // Test write isolation for stack data.
    let test_var = 123 as usize;

    println!(
        "\n\nMain thread test_stack_var_rw, test var is {} at {:#p}",
        test_var, &test_var
    );

    let test_var_addr = &test_var as *const _ as usize;

    std::thread::spawn(move || {
        let test;
        unsafe {
            let test_var = test_var_addr as *mut usize;

            println!(
                "\n\nOn test_thread_read, try to read test var on {:#x}",
                test_var_addr
            );

            test = *test_var;
        }
        println!("\n\nOn test_thread_read, test var is {:#x}", test);
    });

    std::thread::spawn(move || unsafe {
        let test_var = test_var_addr as *mut usize;

        println!(
            "\n\nOn test_thread_write, try to write test var on {:#x}",
            test_var_addr,
        );

        *test_var = 321;

        println!(
            "\n\nOn test_thread_write, test var changed to {:?}",
            test_var.as_mut()
        );
    });
    assert_eq!(test_var, 123, "test_stack_var_rw test_var is modified");
    println!("\n\nBack to main thread, test var changed to {}", test_var);
}

#[allow(unused)]
fn test_global_var_rw() {
    unsafe {
        println!(
            "On main thread, protected global var is {} at {:#p}",
            TEST_PROTECTED_GLOCAL, &TEST_PROTECTED_GLOCAL
        );
        println!(
            "On main thread, shared global var is {} at {:#p}",
            TEST_SHARED_GLOCAL, &TEST_SHARED_GLOCAL
        );
    }

    let mut joinhandles = Vec::new();

    joinhandles.push(std::thread::spawn(move || unsafe {
        println!(
            "On test thread 1, try to read shared global var at {:#p}",
            &TEST_SHARED_GLOCAL
        );
        let global_var = TEST_SHARED_GLOCAL;
        println!(
            "On test thread 1, protected shared var is {} at {:#p}",
            global_var, &TEST_SHARED_GLOCAL
        );
        TEST_SHARED_GLOCAL = 654;
        let global_var = TEST_SHARED_GLOCAL;

        println!(
            "\n\nOn On test thread 1, protected shared var changed to {:?}",
            global_var
        );
    }));

    joinhandles.push(std::thread::spawn(move || unsafe {
        println!(
            "On test thread 2, try to read protected global var at {:#p}",
            &TEST_PROTECTED_GLOCAL
        );
        let global_var = TEST_PROTECTED_GLOCAL;
        println!(
            "On test thread 2, protected global var is {} at {:#p}",
            global_var, &TEST_PROTECTED_GLOCAL
        );
    }));

    for j in joinhandles {
        j.join().unwrap_or_else(|_| {
            println!("The thread being joined has panicked");
        });
    }

    unsafe {
        println!(
            "\n\nBack to main thread, global protected var is {} at {:#p}",
            TEST_PROTECTED_GLOCAL, &TEST_PROTECTED_GLOCAL
        );
        println!(
            "\n\nBack to main thread, global shared var is {} at {:#p}",
            TEST_SHARED_GLOCAL, &TEST_SHARED_GLOCAL
        );
    }
}

#[allow(unused)]
fn test_heap_var_rw() {
    let num_pages = 1;
    use std::mm;
    let shared_addr = mm::allocate(1 << 12 * num_pages);
    let shared_var = shared_addr.as_mut_ptr::<i32>();

    let private_addr = mm::allocate_zone(1 << 12 * num_pages);
    let private_var = private_addr.as_mut_ptr::<i32>();

    unsafe {
        (*shared_var) = 1;
        println!("test_heap_var_rw is {} at {:p}", *shared_var, shared_var);

        (*private_var) = 1;
        println!("test_heap_var_rw is {} at {:p}", *private_var, private_var);
    }

    let shared_addr = shared_var as *const _ as usize;
    let private_addr = private_var as *const _ as usize;

    let mut joinhandles = Vec::new();

    joinhandles.push(std::thread::spawn(move || unsafe {
        let shared_var = shared_addr as *mut i32;
        println!(
            "On test thread 1, try to read shared_var var at {:#p}",
            shared_var
        );
        let test = *shared_var;
        println!(
            "On test thread 1, shared_var is {} at {:#p}",
            test, shared_var
        );
    }));

    joinhandles.push(std::thread::spawn(move || unsafe {
        let private_var = private_addr as *mut i32;

        println!(
            "On test thread 2, try to read private_var var at {:#p}",
            private_var
        );
        let test = *private_var;
        println!(
            "On test thread 2, private_var is {} at {:#p}",
            test, private_var
        );
    }));

    for j in joinhandles {
        j.join().unwrap_or_else(|_| {
            println!("The thread being joined has panicked");
        });
    }
}

// fn test_kernel_var_rw() {

//     let kernel_var =

//     let mut joinhandles = Vec::new();

//     joinhandles.push(std::thread::spawn(move || unsafe {
//         let shared_var = shared_addr as *mut i32;
//         println!(
//             "On test_kernel_var_rw, try to read kernel_var at {:#p}",
//             shared_var
//         );
//         let test = *shared_var;
//         println!(
//             "On test_kernel_var_rw, kernel_var is {} at {:#p}",
//             test, shared_var
//         );
//     }));

//     joinhandles.push(std::thread::spawn(move || unsafe {
//         let private_var = private_addr as *mut i32;

//         println!(
//             "On test thread 2, try to read private_var var at {:#p}",
//             private_var
//         );
//         let test = *private_var;
//         println!(
//             "On test thread 2, private_var is {} at {:#p}",
//             test, private_var
//         );
//     }));

//     for j in joinhandles {
//         j.join().unwrap_or_else(|_| {
//             println!("The thread being joined has panicked");
//         });
//     }
// }

#[no_mangle]
fn main() {
    println!("Hello, world! Unishyper memory isolation bench");

    // test_stack_var_rw();

    // test_global_var_rw();

    test_heap_var_rw();

    println!("Memory isolation bench finished");
}
