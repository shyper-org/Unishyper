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

pub fn swicth_to_kernel_pkru() -> u32 {
    let ori_pkru = rdpkru();
    wrpkru(pkru_of_kernel());
    ori_pkru
}

pub fn switch_from_kernel_pkru(ori_pkru: u32) {
    wrpkru(ori_pkru);
}

const ZONE_ID_MAX: usize = 15;

/// Get intel-MPK zone id according to thread id.
/// By default, zone 0 is reserverd as kernel zone.
/// Thread 100's zone id is 1.
pub fn thread_id_to_zone_id(tid: Tid) -> usize {
    let tid = tid.as_u64() as usize;
    assert!(tid >= 100, "Invalid tid");
    let zone_id = tid - 99;
    if zone_id > ZONE_ID_MAX {
        ZONE_ID_MAX
    } else {
        zone_id
    }
}

pub fn pkru_of_kernel() -> u32 {
    0 as u32
}

pub fn pkru_of_thread_id(tid: usize) -> u32 {
    let zone_id = thread_id_to_zone_id(tid);
    pkru_of_zone_id(zone_id)
}

pub fn pkru_of_zone_id(zone_id: usize) -> u32 {
    if zone_id > 15 {
        return 0;
    }

    let mut pkru = usize::MAX;
    pkru &= !(1 << (zone_id * 2));
    pkru &= !(1 << ((zone_id * 2) + 1));
    pkru as u32
}
