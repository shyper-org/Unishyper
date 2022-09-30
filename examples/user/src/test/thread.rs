use rust_shyper_os::*;

#[allow(dead_code)]
extern "C" fn  switch_back(_: usize) {
    loop {
        thread_yield();
    }
}

#[allow(dead_code)]
pub extern "C" fn  test_thread_switch(_: usize) {
    println!("[TEST] thread ===");
    let _child_thread = thread_spawn_name(switch_back, 1, "switch_back");
    let mut results = vec![];
    for _ in 0..1000 {
        let icntr = current_cycle();
        thread_yield();
        let icntr2 = current_cycle();
        results.push(icntr2 - icntr);
    }
    let mut sum = 0;
    for result in results {
        sum += result;
    }
    println!("[[TEST]] test_thread_switch {}/1000", sum);
    println!("[TEST] thread finished***");
}

