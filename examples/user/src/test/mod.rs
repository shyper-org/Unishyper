mod mm;
mod sem;
mod thread;
mod recover;

use rust_shyper_os::println;
use rust_shyper_os::*;
/// Function and Performance tests for rust-shyperOS.
pub fn run_tests() {
    use rust_shyper_os::*;
    println!("generate_tests:");
    // thread_spawn_bg(mm::test_mm_thread, 1, "mm_test");
    // thread_spawn_bg(thread::test_thread_switch, 1, "thread_test");
    thread_spawn(thread::test_thread_switch, 1);
    // thread_spawn_bg(sem::semaphore_test, 1, "semaphore_test");
    #[cfg(feature = "unwind")]
    thread_spawn_bg(recover::test_recover_thread, 1, "recover_test");
}
