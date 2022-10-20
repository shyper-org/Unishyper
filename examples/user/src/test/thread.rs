use rust_shyper_os::*;

#[allow(dead_code)]
extern "C" fn switch_back(_: usize) {
    irq_disable();
    loop {
        thread_yield();
    }
}

#[allow(dead_code)]
pub extern "C" fn test_thread_switch(_: usize) {
    println!("[TEST] thread ===");
    irq_disable();
    let _child_thread = thread_spawn_name(switch_back, 1, "switch_back");
    let mut results = vec![];
    for i in 0..10010 {
        let icntr = current_cycle();
        thread_yield();
        let icntr2 = current_cycle();
        if i >= 10 {
            results.push(icntr2 - icntr);
        }
        if i % 1000 == 0 || i < 10 {
            println!("round [{}] cycle {}", i, icntr2 - icntr);
        }
    }
    let mut sum = 0;
    for result in results {
        // println!("[{}] result {} cycle", i, result);
        sum += result;
    }
    println!("[[TEST]] test_thread_switch {}/10000", sum);
    println!("[TEST] thread finished***");
}
