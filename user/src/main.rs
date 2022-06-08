#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

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

extern "C" fn test_c_thread(_arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
        "test_c_thread, core {} _arg {} curent EL{}",
        core_id,
        _arg,
        crate::arch::Arch::curent_privilege()
    );
}

#[no_mangle]
fn main() {
    println!("Hello world!\n\nWelcome to shyper lightweight os...\n");

    let tid = thread_spawn(test_c_thread, 1);

    // for i in 0..10 {
    //     let t = crate::lib::thread::thread_alloc(
    //         test_thread as usize,
    //         i as usize,
    //     );
    //     crate::lib::thread::thread_wake(&t);
    // }
    loop {}
}
