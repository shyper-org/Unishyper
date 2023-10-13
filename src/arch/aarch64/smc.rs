#[cfg(not(any(feature = "tx2", feature = "rk3588")))]
#[allow(unused)]
#[cfg(target_arch = "aarch64")]
pub fn smc_call(x0: u64, x1: u64, x2: u64, x3: u64, x4: u64, x5: u64, x6: u64, x7: u64) -> u64 {
    let r;
    unsafe {
        // NOTE: here use hvc for qemu without `virtualization=on`
        core::arch::asm!("hvc #0", inlateout("x0") x0 => r, in("x1") x1, in("x2") x2, in("x3") x3, in("x4") x4, in("x5") x5, in("x6") x6, in("x7") x7);
    }
    r
}

#[cfg(any(feature = "tx2", feature = "rk3588"))]
#[cfg(target_arch = "aarch64")]
pub fn smc_call(x0: u64, x1: u64, x2: u64, x3: u64, x4: u64, x5: u64, x6: u64, x7: u64) -> u64 {
    let r;
    unsafe {
        // NOTE: here use smc for shyper
        core::arch::asm!("smc #0", inlateout("x0") x0 => r, in("x1") x1, in("x2") x2, in("x3") x3, in("x4") x4, in("x5") x5, in("x6") x6, in("x7") x7);
    }
    r
}
