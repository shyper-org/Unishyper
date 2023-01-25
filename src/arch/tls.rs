use cortex_a::registers::{TPIDRRO_EL0, TPIDR_EL0};
use tock_registers::interfaces::{Writeable, Readable};

pub fn set_thread_id(tid: u64) {
    TPIDRRO_EL0.set(tid);
}

#[cfg(target_arch = "aarch64")]
pub fn get_tls_ptr() -> *const u8 {
    // let mut tls_ptr: u64;
    // unsafe {
    //     core::arch::asm!(
    //         "mrs {}, tpidr_el0",
    //         out(reg) tls_ptr,
    //         options(nostack, nomem),
    //     );
    // }
    TPIDR_EL0.get() as *const u8
    // tls_ptr as *const u8
}

#[cfg(target_arch = "aarch64")]
pub fn set_tls_ptr(tls_ptr: u64) {
    TPIDR_EL0.set(tls_ptr);
}
