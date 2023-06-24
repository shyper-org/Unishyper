use std::alloc::{Allocator, Global, Layout};

use super::current_cycle;

const WARNUP_ROUND: usize = 100;
const N_ROUND: usize = 100;

// const SIZE: usize = 2 * 1024 * 1024;
const SIZE: usize = 4096;
const PAGE_SIZE: usize = 4096;

pub fn test_mm_alloc() {
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
