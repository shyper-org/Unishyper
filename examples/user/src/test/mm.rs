use rust_shyper_os::*;

#[allow(dead_code)]
extern "C" fn test_mm_thread(arg: usize) {
    println!("[TEST] memory ===");
    let addr = allocate(1 << 12 * arg);

    let test = addr.as_mut_ptr::<i32>();

    unsafe {
        (*test) = 1;
        println!("test is {}", *test);
    }

    println!(
        "test_mm_thread, region start {:x} size {:x}",
        addr.0,
        1 << 12 * arg
    );

    for i in 10..20 {
        unsafe {
            (*test) = i;
            println!("test is {}", *test);
        }
    }
    println!("[TEST] memory finished***");
}

#[allow(dead_code)]
pub fn mm_test() {
    thread_spawn(test_mm_thread, 1);
}
