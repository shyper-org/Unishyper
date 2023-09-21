//! This module contains hypervisor call(HVC) definition.
//!
//! Specifically, the HVC is designed for shyper, and can only be run on shyper.

/// HVC fid for Unilib operation.
pub const HVC_UNILIB: usize = 0x12;

/* HVC events for Unilib-fs opeation*/
pub const HVC_UNILIB_FS_INIT: usize = 0;
pub const HVC_UNILIB_FS_OPEN: usize = 1;
pub const HVC_UNILIB_FS_CLOSE: usize = 2;
pub const HVC_UNILIB_FS_READ: usize = 3;
pub const HVC_UNILIB_FS_WRITE: usize = 4;
pub const HVC_UNILIB_FS_LSEEK: usize = 5;
pub const HVC_UNILIB_FS_STAT: usize = 6;
pub const HVC_UNILIB_FS_UNLINK: usize = 7;
#[allow(dead_code)]
pub const HVC_UNILIB_FS_APPEND: usize = 0x10;
#[allow(dead_code)]
pub const HVC_UNILIB_FS_FINISHED: usize = 0x11;

#[macro_export]
macro_rules! hvc_mode {
    ($fid: expr, $event: expr) => {
        ((($fid << 8) | $event) & 0xffff)
    };
}

/// HVC API for operating the hvc request to shyper.
/// During this function, current core will fall into EL2 mode and perform operation inside hypervisor.
/// ## Arguments
/// * `x0-x2`       - The intermediated physical address of the path that GVM wants to open through unilib-fs API.
/// * `hvc_mode`    - The fid and event of this HVC call, it's a 64 bit value,
///                     bits (63:32) is hvc_fid, bits (31: 0) is hvc_event, you can use the macro `hvc_mode` for transfer.                .
/// ## Return value
/// * Return hvc call result from hypervisor pass through x0 register.
pub fn hvc_call(x0: usize, x1: usize, x2: usize, hvc_mode: usize) -> u64 {
    #[cfg(target_arch = "aarch64")]
    crate::arch::smc::smc_call(x0 as u64, x1 as u64, x2 as u64, 0, 0, 0, 0, hvc_mode as u64)
}
