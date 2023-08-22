use core::arch::asm;

/// Get current PKRU register value.
#[inline]
pub fn rdpkru() -> u32 {
    let val: u32;
    unsafe {
        asm!(
            "xor ecx, ecx",
            // https://shell-storm.org/x86doc/RDPKRU.html
            "rdpkru",
            lateout("eax") val,
        );
    }
    val
}

/// Set current PKRU register value.
#[inline]
pub fn wrpkru(val: u32) {
    unsafe {
        asm!(
            "xor ecx, ecx",
            "xor edx, edx",
            // https://www.felixcloutier.com/x86/wrpkru
            "wrpkru",
            // Performs a serializing operation on all load-from-memory instructions
            // that were issued prior the LFENCE instruction.
            // https://www.felixcloutier.com/x86/lfence
            "lfence",
            in("eax") val,
        );
    }
}



