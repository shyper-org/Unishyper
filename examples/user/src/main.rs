#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use rust_shyper_os::arch::*;
use rust_shyper_os::exported::*;
use rust_shyper_os::*;

// #[no_mangle]
// fn test_thread(_arg: usize) {
//     let core_id = crate::arch::Arch::core_id();
//     println!(
//         "test_thread, core {} _arg {} curent EL{}",
//         core_id,
//         _arg,
//         crate::arch::Arch::curent_privilege()
//     );
//     exit();
// }

// extern "C" fn test_c_thread(arg: usize) {
//     let core_id = crate::arch::Arch::core_id();
//     println!(
//         "test_c_thread, core {} arg {} curent EL{}",
//         core_id,
//         arg,
//         crate::arch::Arch::curent_privilege()
//     );
// }

// extern "C" fn test_mm_thread(arg: usize) {
//     let core_id = crate::arch::Arch::core_id();
//     println!(
//         "test_mm_thread, core {} arg {} curent EL{}\n",
//         core_id,
//         arg,
//         crate::arch::Arch::curent_privilege()
//     );
//     let addr = allocate(PAGE_SIZE * 2);

//     let test = addr.as_mut_ptr::<i32>();

//     unsafe {
//         (*test) = 1;
//         println!("test is {}", *test);
//     }

//     println!(
//         "test_mm_thread, region start {:x} size {:x}",
//         addr.0,
//         PAGE_SIZE * 2
//     );

//     for i in 10..20 {
//         unsafe {
//             (*test) = i;
//             println!("test is {}", *test);
//         }
//     }
// }

// use rust_shyper_os::lib::thread::thread_yield;
// extern "C" fn test_yield_thread_1(arg: usize) {
//     let core_id = crate::arch::Arch::core_id();
//     loop {
//         println!(
//             "\n==========================\ntest_yield_thread_1, core {} arg {} curent EL{}\n==========================\n",
//             core_id,
//             arg,
//             crate::arch::Arch::curent_privilege()
//         );
//         thread_yield();
//     }
// }

// extern "C" fn test_yield_thread_2(arg: usize) {
//     let core_id = crate::arch::Arch::core_id();
//     loop {
//         println!(
//             "\n**************************\ntest_yield_thread_2, core {} arg {} curent EL{}\n**************************\n",
//             core_id,
//             arg,
//             crate::arch::Arch::curent_privilege()
//         );
//         thread_yield();
//     }
// }



use rust_shyper_os::exported::semaphore::Semaphore;

static TEST_SEM: Semaphore = Semaphore::new(0);

#[allow(dead_code)]
extern "C" fn test_semaphore_acquire(arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
        "\n**************************\n test_semaphore_acquire, core {} arg {} curent EL{}\n**************************\n",
        core_id,
        arg,
        crate::arch::Arch::curent_privilege()
    );
    let mut i = 0;
    loop {
        println!("\n[Acquire Thread] acquire round {}\n", i);
        TEST_SEM.acquire();
        println!("\n[Acquire Thread] acquired success on round {}\n", i);
        i += 1;
    }
}

#[allow(dead_code)]
extern "C" fn test_semaphore_release_A(arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
        "\n**************************\n test_semaphore_release, core {} arg {} curent EL{}\n**************************\n",
        core_id,
        arg,
        crate::arch::Arch::curent_privilege()
    );
    for i in 0..arg {
        println!("\n[Release Thread A] release round {}\n", i);
        TEST_SEM.release();
        thread_block_current_with_timeout(2000);
        thread_yield();
    }
}

#[allow(dead_code)]
extern "C" fn test_semaphore_release_B(arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
        "\n**************************\n test_semaphore_release, core {} arg {} curent EL{}\n**************************\n",
        core_id,
        arg,
        crate::arch::Arch::curent_privilege()
    );
    for i in 0..arg {
        println!("\n[Release Thread B] release round {}\n", i);
        TEST_SEM.release();
        thread_block_current_with_timeout(1000);
        thread_yield();
    }
}

#[no_mangle]
fn main() {
    // thread_spawn(test_mm_thread, 321);
    // thread_spawn(network_init, 0);

    // thread_spawn(test_net_sem, 1);
    thread_spawn(test_semaphore_acquire, 123);
    // // thread_yield();
    thread_spawn(test_semaphore_release_A, 3);
    thread_spawn(test_semaphore_release_B, 5);
    thread_yield();
    // for i in 0..10 {
    //     thread_spawn(test_c_thread, i + 100);
    // }
    // use rust_shyper_os::lib::thread::thread_yield;
    // thread_yield();
    exit();
}
