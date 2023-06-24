mod mm;
// mod sem;
mod thread;

#[cfg(target_arch = "aarch64")]
pub fn current_cycle() -> usize {
	let r;
	unsafe {
		core::arch::asm!("mrs {}, pmccntr_el0", out(reg) r);
	}
	r
}

#[cfg(target_arch = "x86_64")]
pub fn current_cycle() -> usize {
	unsafe {
		core::arch::x86_64::_mm_lfence();
		let value = core::arch::x86_64::_rdtsc();
		core::arch::x86_64::_mm_lfence();
		value as usize
	}
}

/// Function and Performance tests for rust-shyperOS.
pub fn run_tests() {
    println!("generate_tests:");
    // thread::test_thread_switch();
    // thread::test_thread_spawn();
    mm::test_mm_alloc();
}
