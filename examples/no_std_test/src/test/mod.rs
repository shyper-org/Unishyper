mod mm;
// mod sem;
mod recover;
mod schedule;
mod thread;

use unishyper::println;
use unishyper::*;
/// Function and Performance tests for rust-shyperOS.
pub fn run_tests() {
    use unishyper::*;
    println!("generate_tests:");
    // thread_spawn_bg(mm::test_mm_thread, 1, "mm_test");
    // thread_spawn_bg(thread::test_thread_switch, 1, "thread_test");
    // thread_spawn(thread::test_thread_switch, 1);
    // thread_spawn(thread::test_thread_create, 1);
    // thread_spawn(mm::test_allocator_thread, 1);
    thread_spawn(mm::test_mm_thread, 1);
    // thread_spawn_bg(sem::semaphore_test, 1, "semaphore_test");
    // thread_spawn(thread::test_thread_getid, 1);
    // thread_spawn(schedule::test_thread_schedule, 4);
    // #[cfg(feature = "unwind")]
    // thread_spawn(recover::test_recover_thread, 1);
    // thread_spawn_bg(recover::test_recover_thread, 1, "recover_test");
}
