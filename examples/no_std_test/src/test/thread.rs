use unishyper::*;

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
    let mut min = usize::MAX;
    for i in 0..20000 {
        let icntr = current_cycle();
        thread_yield();
        let icntr2 = current_cycle();

        if icntr2 - icntr < min {
            min = icntr2 - icntr;
        }

        if i >= 10000 {
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
    println!("[[TEST]] test_thread_switch {}/10000, min {}", sum, min);
    println!("[TEST] thread finished***");
}

#[allow(dead_code)]
pub extern "C" fn test_thread_getid(_: usize) {
    println!("[TEST] thread get pid test begin ===");
    irq_disable();
    let mut results = vec![];
    for i in 0..10010 {
        let icntr = current_cycle();
        let _ = current_thread_id();
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
    println!("[[TEST]] test_thread_getid {}/10000", sum);
    println!("[TEST] thread get pid test finished***");
}

pub extern "C" fn test_created_thread(_: usize) {
    // thread_yield();
    // thread_exit();
}

#[allow(dead_code)]
pub extern "C" fn test_thread_create(_: usize) {
    println!("[TEST] thread create test begin ===");
    irq_disable();
    let mut results = vec![];
    let mut min = usize::MAX;
    for i in 0..20000 {
        // println!("[TEST] thread_spawn");
        irq_disable();
        let icntr = current_cycle();
        let tid = thread_spawn(test_created_thread, 1);
        // thread_yield();
        let icntr2 = current_cycle();
        // println!("[TEST] thread_spawn end");
        thread_destroy_by_tid(tid);
        thread_yield();
        if icntr2 - icntr < min {
            min = icntr2 - icntr;
        }
        if i >= 10000 {
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
    println!("[[TEST]] test_thread_create {}/10000 min {}", sum, min);
    println!("[TEST] test_thread_create test finished***");
}
