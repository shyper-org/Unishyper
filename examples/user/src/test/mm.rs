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

use core::alloc::Layout;
use core::alloc::{GlobalAlloc, Allocator};

use unishyper::Global;

const WARNUP_ROUND: usize = 100;
const N_ROUND: usize = 100;

const SIZE: usize = 4096;
const PAGE_SIZE: usize = 4096;

pub extern "C" fn test_allocator_thread(_: usize) {
    let mut results = vec![];
    let mut results_1 = vec![];

    for i in 0..N_ROUND + WARNUP_ROUND {
        // let layout = Layout::new::<i32>();
        let layout = Layout::from_size_align(SIZE, PAGE_SIZE).unwrap();
        let cnt: usize;
        let cnt_1: usize;
        unsafe {
            let start = current_cycle();
            let res = Global.allocate(layout);
            let end = current_cycle();

            cnt = end - start;

            // println!("{:?}", res);
            // if i % 1000 == 0 || i < 10 {
            // 	println!("{:?}", res);
            // }
            let ptr = res.unwrap();

            let start = current_cycle();
            Global.deallocate(ptr.cast(), layout);
            let end = current_cycle();

            cnt_1 = end - start;
        }

        if i >= WARNUP_ROUND {
            results.push(cnt);
            results_1.push(cnt_1);
        }
        if i % 1000 == 0 || i < 10 {
            println!(
                "main thread, round [{}] allocate {} cycles deallocate {} cycles",
                i, cnt, cnt_1
            );
        }
    }

    let mut sum = 0;
    for result in results {
        // println!("[{}] result {} cycle", i, result);
        sum += result;
    }

    let mut sum_1 = 0;
    for result in results_1 {
        // println!("[{}] result {} cycle", i, result);
        sum_1 += result;
    }

    println!("[[TEST]] test_mm allocate {}/{N_ROUND}", sum);
    println!("[[TEST]] test_mm deallocate {}/{N_ROUND}", sum_1);
    println!("[TEST] test_mm_alloc finished***");
}

pub extern "C" fn test_mm_alloc(_: usize) {
	let mut results = vec![];
	let mut results_1 = vec![];
	for i in 0..N_ROUND + WARNUP_ROUND {
		let layout = Layout::from_size_align(SIZE, PAGE_SIZE).unwrap();
		// let layout = Layout::new::<i32>();
		let mut res_list = vec![];
		let cnt: usize;
        let cnt_1: usize;
		let mut cnt_inside = 0;
		for _j in 0..100 {
			unsafe {
				let start = current_cycle();
				// let res = Global.allocate(layout);
				let res = Global.allocate_zeroed(layout);
				let end = current_cycle();
				res_list.push(res);
				cnt_inside = cnt_inside + (end - start);
			}
		}
		cnt = cnt_inside / 100;
		println!("main thread, round [{}] allocate {} cycles", i, cnt);
        cnt_inside = 0;
		for res in res_list {
			let ptr = res.unwrap();
			unsafe {
				let start = current_cycle();
				Global.deallocate(ptr.cast(), layout);
				let end = current_cycle();
                cnt_inside = cnt_inside + (end - start);
			}
		}
        cnt_1 = cnt_inside / 100;
		println!("main thread, round [{}] deallocate {} cycles", i, cnt_1);

		if i >= WARNUP_ROUND {
			results.push(cnt);
			results_1.push(cnt_1);
		}
		if i % 1000 == 0 || i < 10 {
			println!(
				"main thread, round [{}] allocate {} cycles deallocate {} cycles",
				i, cnt, cnt_1
			);
		}
	}

	let mut sum = 0;
	for result in results {
		// println!("[{}] result {} cycle", i, result);
		sum += result;
	}

	let mut sum_1 = 0;
	for result in results_1 {
		// println!("[{}] result {} cycle", i, result);
		sum_1 += result;
	}

	println!("[[TEST]] test_mm allocate {}/{N_ROUND}", sum);
	println!("[[TEST]] test_mm deallocate {}/{N_ROUND}", sum_1);
	println!("[TEST] test_mm_alloc finished***");
}