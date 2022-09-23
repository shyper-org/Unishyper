use crate::mm::address::VAddr;

pub fn allocate(size: usize) -> VAddr {
    match crate::mm::allocate(size) {
        Some(addr) => {
            return addr;
        },
        None => {
            error!("failed to allocate memory of size {}", size);
            return VAddr::zero();
        }
    }
}

pub fn deallocate(address: VAddr) {
    crate::mm::deallocate(address);
}