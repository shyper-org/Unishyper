use unishyper::*;

#[allow(dead_code)]
pub extern "C" fn test_mm_thread(arg: usize) {
    println!("[TEST] memory ===");
    let addr = allocate(1 << 12 * arg);

    let test = addr.as_mut_ptr::<i32>();

    unsafe {
        (*test) = 1;
        println!("test is {} at {:p}", *test, test);
    }

    println!(
        "test_mm_thread, region start {} size 0x{:x}",
        addr,
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
