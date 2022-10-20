use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use bit_field::BitField;
use zerocopy::FromBytes;

use crate::arch::PAGE_SIZE;
// use crate::libs::traits::Address;
use crate::mm::interface::MapGranularity;

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Binary,
    Octal,
    LowerHex,
    UpperHex,
    BitAnd,
    BitOr,
    BitXor,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    FromBytes,
)]
#[repr(transparent)]
pub struct VAddr(usize);

impl VAddr {
    pub fn new(addr: usize) -> Option<VAddr> {
        if is_canonical_virtual_address(addr) {
            Some(VAddr(addr))
        } else {
            None
        }
    }

    pub const fn new_canonical(addr: usize) -> VAddr {
        VAddr(canonicalize_virtual_address(addr))
    }

    pub const fn zero() -> VAddr {
        VAddr(0)
    }

    #[inline]
    pub const fn value(&self) -> usize {
        self.0
    }

    pub const fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    pub const fn page_offset_2mb(&self) -> usize {
        self.0 & (MapGranularity::Page2MB as usize - 1)
    }

    // Todo: remove this method.
    pub fn to_physical_address(&self) -> PAddr {
        crate::mm::paging::virt_to_phys(&self)
    }

    pub fn is_kernel_address(&self) -> bool {
        self.0.get_bit(63)
    }

    /// Convert to mutable pointer.
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    /// Convert to pointer.
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }
}

impl fmt::Debug for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, concat!("VAddr: ", "0x{:016x}"), self.0)
    }
}

impl fmt::Display for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Pointer for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Add<usize> for VAddr {
    type Output = VAddr;
    fn add(self, rhs: usize) -> VAddr {
        VAddr::new_canonical(self.0.saturating_add(rhs))
    }
}

impl AddAssign<usize> for VAddr {
    fn add_assign(&mut self, rhs: usize) {
        *self = VAddr::new_canonical(self.0.saturating_add(rhs));
    }
}

impl Sub<usize> for VAddr {
    type Output = VAddr;
    fn sub(self, rhs: usize) -> VAddr {
        VAddr::new_canonical(self.0.saturating_sub(rhs))
    }
}

impl SubAssign<usize> for VAddr {
    fn sub_assign(&mut self, rhs: usize) {
        *self = VAddr::new_canonical(self.0.saturating_sub(rhs));
    }
}

impl From<usize> for VAddr {
    fn from(addr: usize) -> Self {
        VAddr(addr)
    }
}

impl Into<usize> for VAddr {
    #[inline]
    fn into(self) -> usize {
        self.0
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Binary,
    Octal,
    LowerHex,
    UpperHex,
    BitAnd,
    BitOr,
    BitXor,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    FromBytes,
)]
#[repr(transparent)]
pub struct PAddr(usize);

impl PAddr {
    pub fn new(addr: usize) -> Option<PAddr> {
        if is_canonical_physical_address(addr) {
            Some(PAddr(addr))
        } else {
            None
        }
    }

    pub const fn new_canonical(addr: usize) -> PAddr {
        PAddr(canonicalize_physical_address(addr))
    }

    pub const fn zero() -> PAddr {
        PAddr(0)
    }

    #[inline]
    pub const fn value(&self) -> usize {
        self.0
    }

    pub const fn frame_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}

impl fmt::Debug for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, concat!("PAddr: ", "0x{:016x}"), self.0)
    }
}

impl fmt::Display for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Pointer for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Add<usize> for PAddr {
    type Output = PAddr;
    fn add(self, rhs: usize) -> PAddr {
        PAddr::new_canonical(self.0.saturating_add(rhs))
    }
}

impl AddAssign<usize> for PAddr {
    fn add_assign(&mut self, rhs: usize) {
        *self = PAddr::new_canonical(self.0.saturating_add(rhs));
    }
}

impl Sub<usize> for PAddr {
    type Output = PAddr;
    fn sub(self, rhs: usize) -> PAddr {
        PAddr::new_canonical(self.0.saturating_sub(rhs))
    }
}

impl SubAssign<usize> for PAddr {
    fn sub_assign(&mut self, rhs: usize) {
        *self = PAddr::new_canonical(self.0.saturating_sub(rhs));
    }
}

impl Into<usize> for PAddr {
    #[inline]
    fn into(self) -> usize {
        self.0
    }
}

#[inline]
fn is_canonical_virtual_address(virt_addr: usize) -> bool {
    match virt_addr.get_bits(47..64) {
        0 | 0b1_1111_1111_1111_1111 => true,
        _ => false,
    }
}

#[inline]
const fn canonicalize_virtual_address(virt_addr: usize) -> usize {
    // match virt_addr.get_bit(47) {
    //     false => virt_addr.set_bits(48..64, 0),
    //     true =>  virt_addr.set_bits(48..64, 0xffff),
    // };

    // The below code is semantically equivalent to the above, but it works in const functions.
    ((virt_addr << 16) as isize >> 16) as usize
}

#[inline]
fn is_canonical_physical_address(phys_addr: usize) -> bool {
    match phys_addr.get_bits(52..64) {
        0 => true,
        _ => false,
    }
}

#[inline]
const fn canonicalize_physical_address(phys_addr: usize) -> usize {
    phys_addr & 0x000F_FFFF_FFFF_FFFF
}
