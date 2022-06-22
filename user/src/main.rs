#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

use rust_shyper_os::arch::*;
use rust_shyper_os::exported::*;
use rust_shyper_os::*;

#[no_mangle]
fn test_thread(_arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
        "test_thread, core {} _arg {} curent EL{}",
        core_id,
        _arg,
        crate::arch::Arch::curent_privilege()
    );
    exit();
}

extern "C" fn test_c_thread(arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
        "test_c_thread, core {} arg {} curent EL{}",
        core_id,
        arg,
        crate::arch::Arch::curent_privilege()
    );
}

extern "C" fn test_mm_thread(arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
        "test_mm_thread, core {} arg {} curent EL{}\n",
        core_id,
        arg,
        crate::arch::Arch::curent_privilege()
    );
    let addr = allocate(PAGE_SIZE * 2);

    let test = addr.as_mut_ptr::<i32>();

    unsafe {
        (*test) = 1;
        println!("test is {}", *test);
    }

    println!(
        "test_mm_thread, region start {:x} size {:x}",
        addr.0,
        PAGE_SIZE * 2
    );

    for i in 10..20 {
        unsafe {
            (*test) = i;
            println!("test is {}", *test);
        }
    }
}

#[no_mangle]
fn main() {
    println!("Hello world!\n\nWelcome to shyper lightweight os...\n");

    network_init();

    let tid = thread_spawn(test_mm_thread, 1);

    // for i in 0..10 {
    //     thread_spawn(test_c_thread, i + 100);
    // }
    loop {}
}
