use core::arch::asm;

/// Get current PKRU register value.
/// !!! `ecx` is modified inside unsafe block.
#[inline(never)]
pub fn rdpkru() -> u32 {
    let val: u32;
    unsafe {
        asm!(
            // "push rcx",
            "xor ecx, ecx",
            // https://shell-storm.org/x86doc/RDPKRU.html
            "rdpkru",
            // "pop rcx",
            lateout("eax") val,
        );
    }
    val
}

/// Set current PKRU register value.
/// !!! `ecx` and `edx` are modified inside unsafe block.
#[inline(never)]
pub fn wrpkru(val: u32) {
    unsafe {
        asm!(
            // "push rcx",
            // "push rdx",
            "xor ecx, ecx",
            "xor edx, edx",
            // https://www.felixcloutier.com/x86/wrpkru
            "wrpkru",
            // Performs a serializing operation on all load-from-memory instructions
            // that were issued prior the LFENCE instruction.
            // https://www.felixcloutier.com/x86/lfence
            "lfence",
            // "pop rdx",
            // "pop rcx",
            in("eax") val,
        );
    }
}
