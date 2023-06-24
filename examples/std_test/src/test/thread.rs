use std::thread;

use super::current_cycle;

const WARNUP_ROUND: usize = 10000;
const N_ROUND: usize = 10000;

pub fn test_thread_spawn() {
	let mut results = vec![];
	for i in 0..N_ROUND + WARNUP_ROUND {
		let start = current_cycle();
		let _handle = thread::spawn(|| {});
		let cnt = current_cycle() - start;
		drop(_handle);
		thread::yield_now();

		if i >= WARNUP_ROUND {
			results.push(cnt);
		}
		if i % 1000 == 0 || i < 10 {
			println!(
				"main thread, round [{}] {} cycles start {} end {}",
				i,
				cnt,
				start,
				start + cnt
			);
		}
	}

	let mut sum = 0;
	for result in results {
		// println!("[{}] result {} cycle", i, result);
		sum += result;
	}

	println!("[[TEST]] test_thread_spawn {}/{N_ROUND}", sum);
	println!("[TEST] thread finished***");
}

pub fn test_thread_switch() {
	let _handle = thread::spawn(|| loop {
		thread::yield_now();
	});

	let mut results = vec![];
	for i in 0..N_ROUND + WARNUP_ROUND {
		let start = current_cycle();
		thread::yield_now();
		let cnt = current_cycle() - start;
		if i >= WARNUP_ROUND {
			results.push(cnt);
		}
		if i % 1000 == 0 || i < 10 {
			println!(
				"main thread, round [{}] {} cycles start {} end {}",
				i,
				cnt,
				start,
				start + cnt
			);
		}
	}

	let mut sum = 0;
	for result in results {
		// println!("[{}] result {} cycle", i, result);
		sum += result;
	}

	println!("[[TEST]] test_thread_switch {}/{N_ROUND}", sum);
	println!("[TEST] thread finished***");
}
