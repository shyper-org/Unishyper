mod mm;
mod sem;
mod thread;

use rust_shyper_os::println;
/// Function and Performance tests for rust-shyperOS.
pub fn run_tests() {
    use rust_shyper_os::*;
    println!("run_tests");
    // mm::mm_test();
    thread::thread_test();
    // sem::semaphore_test();
}
