use rust_shyper_os::*;
use rust_shyper_os::semaphore::Semaphore;

static TEST_SEM: Semaphore = Semaphore::new(0);

#[allow(dead_code)]
extern "C" fn test_semaphore_acquire(_arg: usize) {
    let mut i = 0;
    loop {
        println!("\n[Acquire Thread] acquire round {}\n", i);
        TEST_SEM.acquire();
        println!("\n[Acquire Thread] acquired success on round {}\n", i);
        i += 1;
    }
}

#[allow(dead_code)]
extern "C" fn test_semaphore_release_a(arg: usize) {
    for i in 0..arg {
        println!("\n[Release Thread A] release round {}\n", i);
        TEST_SEM.release();
        thread_yield();
    }
}

#[allow(dead_code)]
extern "C" fn test_semaphore_release_b(arg: usize) {
    for i in 0..arg {
        println!("\n[Release Thread B] release round {}\n", i);
        TEST_SEM.release();
        thread_yield();
    }
}

#[allow(dead_code)]
pub fn semaphore_test() {
    println!("[TEST] === semaphore ===");
    thread_spawn(test_semaphore_acquire, 1);
    thread_spawn(test_semaphore_release_a, 2);
    thread_spawn(test_semaphore_release_b, 3);
}