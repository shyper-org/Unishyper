use rust_shyper_os::*;

#[allow(dead_code)]
extern "C" fn test_spawned_thread(index: usize) {
    println!(
        "------------ test_spawned_thread [{}] on core [{}]",
        index,
        core_id()
    );
    loop {}
}

#[allow(dead_code)]
pub extern "C" fn test_thread_schedule(_: usize) {
    println!("[TEST] test_thread_schedule ===");
    irq_disable();
    for i in 0..10 {
        let tid = thread_spawn_on_core(test_spawned_thread, i, 1);
        let tid = thread_spawn(test_spawned_thread, i * 10 + 1);
        println!("[{}] Thread [{}] spawned success", i, tid);
    }
    println!("[TEST] test_thread_schedule spawn finish");
}
