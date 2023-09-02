#![no_std]

#[cfg(target_arch = "x86_64")]
mod pkey;

#[cfg(target_arch = "x86_64")]
pub use pkey::{ZoneKeys, ZONE_ID_SHARED};

pub type ZoneId = usize;

mod arch;
pub use arch::*;

// pub const fn zone_protected() -> ZoneId {
//     #[cfg(target_arch = "x86_64")]
//     {
//         pkey::zone_protected()
//     }
//     #[cfg(not(target_arch = "x86_64"))]
//     0
// }

pub fn zone_init() {
    #[cfg(target_arch = "x86_64")]
    pkey::zone_init();
}

pub fn zone_alloc() -> Option<ZoneId> {
    #[cfg(target_arch = "x86_64")]
    {
        pkey::zone_alloc()
    }
    #[cfg(not(target_arch = "x86_64"))]
    None
}

pub fn zone_free(_zone_id: ZoneId) {
    #[cfg(target_arch = "x86_64")]
    {
        pkey::zone_free(_zone_id);
    }
}

pub fn switch_to_privilege() -> ZoneId {
    #[cfg(target_arch = "x86_64")]
    {
        pkey::switch_to_privilege() as ZoneId
    }
    #[cfg(not(target_arch = "x86_64"))]
    0
}

pub fn switch_from_privilege(_zone_id: ZoneId) {
    #[cfg(target_arch = "x86_64")]
    pkey::switch_from_privilege(_zone_id as u32);
}

pub fn protected_function_wrapper<F>(f: F)
where
    F: FnOnce() -> (),
{
    let ori_zone = switch_to_privilege();

    f();

    switch_from_privilege(ori_zone);
}
