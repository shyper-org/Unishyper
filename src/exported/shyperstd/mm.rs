use crate::mm::address::VAddr;

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn allocate(size: usize) -> VAddr {
    match crate::mm::allocate(size, false) {
        Some(addr) => addr,
        None => {
            error!("failed to allocate memory of size {}", size);
            VAddr::zero()
        }
    }
}

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn allocate_zone(size: usize) -> VAddr {
    match crate::mm::allocate(size, true) {
        Some(addr) => addr,
        None => {
            error!("failed to allocate memory of size {}", size);
            VAddr::zero()
        }
    }
}

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn deallocate(address: VAddr) {
    crate::mm::deallocate(address);
}
