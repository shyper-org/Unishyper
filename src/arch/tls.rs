#[cfg(target_arch = "aarch64")]
pub fn get_tls_ptr() -> *const u8 {
    let mut tls_ptr: u64;
    unsafe {
        core::arch::asm!(
			"mrs {}, tpidr_el0",
			out(reg) tls_ptr,
			options(nostack, nomem),
		);
    }
    tls_ptr as *const u8
}

#[cfg(target_arch = "aarch64")]
pub fn set_tls_ptr(tls_ptr: u64){
    unsafe {
        // NOTE: here use hvc for qemu without `virtualization=on`
        core::arch::asm!(
			"msr tpidr_el0, {}",
			in(reg) tls_ptr,
			options(nostack, nomem),
		);
    }
}