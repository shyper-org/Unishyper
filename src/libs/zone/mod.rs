#[cfg(not(feature = "zone"))]
mod dummy;

#[cfg(feature = "zone")]
mod pkey;

#[cfg(not(feature = "zone"))]
pub use dummy::*;

#[cfg(feature = "zone")]
pub use pkey::*;

#[cfg(feature = "zone")]
pub fn protected_function_wrapper<F>(f: F)
where
    F: FnOnce() -> (),
{
    let ori_pkru = rdpkru();
    // println!("enter protection function, current pkru {:#x}", ori_pkru);
    wrpkru(PKRU_PRIVILEGED);

    f();
    
    wrpkru(ori_pkru);
}

#[cfg(not(feature = "zone"))]
pub fn protected_function_wrapper<F>(f: F)
where
    F: FnOnce() -> (),
{
    f();
}