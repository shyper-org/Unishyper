use alloc::boxed::Box;

use unishyper::*;

struct ResourceA;
struct ResourceB;

impl Drop for ResourceA {
    fn drop(&mut self) {
        println!("Resource A drop here");
    }
}

impl Drop for ResourceB {
    fn drop(&mut self) {
        println!("Resource B drop here");
    }
}

#[allow(dead_code)]
extern "C" fn test_panic(arg: usize) {
    println!("test_panic thread arg {}", arg);
    let _a = Box::new(ResourceA);
    panic!("Simulate a panic!");
    // let _b = Box::new(ResourceB);
}

#[allow(dead_code)]
extern "C" fn test_page_fault(arg: usize) {
    println!("test_page_fault thread arg {}", arg);
    let _a = Box::new(ResourceA);
    unsafe {
        (0xdeafbeef0000 as *mut usize).write(0);
    }
    let _b = Box::new(ResourceB);
}

#[inject::panic_inject]
#[inject::count_stmts]
#[allow(dead_code)]
pub extern "C" fn test_inject_thread(_arg: usize) {
    println!("test_inject_thread");
    loop {
        // println!("[LOOP] test_inject_thread");
    }
}

pub extern "C" fn test_recover(arg: usize) {
    println!("[TEST] === test_recover ===");
    thread_spawn(test_panic, arg);
    thread_spawn(test_page_fault, arg);
    // thread_spawn(test_inject_thread, arg);
}
