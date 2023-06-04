//! A module containing all environment specific function calls.
//!
//! The module should easy partability of the code. Furthermore it provides
//! a clean boundary between virtio and the rest of the kernel. One additional aspect is to
//! ensure only a single location needs changes, in cases where the underlying kernel code is changed

pub mod memory {
    use core::ops::Add;

    /// A newtype representing a memory offset which can be used to be added to [PhyMemAddr](PhyMemAddr) or
    /// to [VirtMemAddr](VirtMemAddr).
    #[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct MemOff(usize);

    // INFO: In case Offset is change to supporrt other than 64 bit systems one also needs to adjust
    // the respective From<Offset> for u32 implementation.
    impl From<u32> for MemOff {
        fn from(val: u32) -> Self {
            MemOff(usize::try_from(val).unwrap())
        }
    }

    impl From<u64> for MemOff {
        fn from(val: u64) -> Self {
            MemOff(usize::try_from(val).unwrap())
        }
    }

    impl From<MemOff> for u32 {
        fn from(val: MemOff) -> u32 {
            u32::try_from(val.0).unwrap()
        }
    }

    /// A newtype representing a memory length which can be used to be added to [PhyMemAddr](PhyMemAddr) or
    /// to [VirtMemAddr](VirtMemAddr).
    #[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct MemLen(usize);

    // INFO: In case Offset is change to supporrt other than 64 bit systems one also needs to adjust
    // the respective From<Offset> for u32 implementation.
    impl From<u32> for MemLen {
        fn from(val: u32) -> Self {
            MemLen(usize::try_from(val).unwrap())
        }
    }

    impl From<u64> for MemLen {
        fn from(val: u64) -> Self {
            MemLen(usize::try_from(val).unwrap())
        }
    }

    impl From<usize> for MemLen {
        fn from(val: usize) -> Self {
            MemLen(val)
        }
    }

    impl From<MemLen> for usize {
        fn from(val: MemLen) -> usize {
            val.0
        }
    }

    impl From<MemLen> for u32 {
        fn from(val: MemLen) -> u32 {
            u32::try_from(val.0).unwrap()
        }
    }

    impl From<MemLen> for u64 {
        fn from(val: MemLen) -> u64 {
            u64::try_from(val.0).unwrap()
        }
    }

    impl MemLen {
        pub fn from_rng(start: VirtMemAddr, end: MemOff) -> MemLen {
            MemLen(start.0 + end.0)
        }
    }

    impl Add for MemLen {
        type Output = MemLen;

        fn add(self, other: Self) -> Self::Output {
            MemLen(self.0 + other.0)
        }
    }

    impl Add<MemOff> for MemLen {
        type Output = MemLen;

        fn add(self, other: MemOff) -> MemLen {
            MemLen(self.0 + other.0)
        }
    }

    /// A newtype representing a virtual mempory address.
    #[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct VirtMemAddr(usize);

    impl From<u32> for VirtMemAddr {
        fn from(addr: u32) -> Self {
            VirtMemAddr(usize::try_from(addr).unwrap())
        }
    }

    impl From<u64> for VirtMemAddr {
        fn from(addr: u64) -> Self {
            VirtMemAddr(usize::try_from(addr).unwrap())
        }
    }

    impl From<usize> for VirtMemAddr {
        fn from(addr: usize) -> Self {
            VirtMemAddr(addr)
        }
    }

    impl From<VirtMemAddr> for usize {
        fn from(addr: VirtMemAddr) -> usize {
            addr.0
        }
    }

    impl Add<MemOff> for VirtMemAddr {
        type Output = VirtMemAddr;

        fn add(self, other: MemOff) -> Self::Output {
            VirtMemAddr(self.0 + other.0)
        }
    }

    /// A newtype representing a physical memory address
    pub struct PhyMemAddr(usize);

    impl From<u32> for PhyMemAddr {
        fn from(addr: u32) -> Self {
            PhyMemAddr(usize::try_from(addr).unwrap())
        }
    }

    impl From<u64> for PhyMemAddr {
        fn from(addr: u64) -> Self {
            PhyMemAddr(usize::try_from(addr).unwrap())
        }
    }

    impl From<PhyMemAddr> for usize {
        fn from(addr: PhyMemAddr) -> usize {
            addr.0
        }
    }

    impl From<usize> for PhyMemAddr {
        fn from(addr: usize) -> Self {
            PhyMemAddr(addr)
        }
    }

    impl Add<MemOff> for PhyMemAddr {
        type Output = PhyMemAddr;

        fn add(self, other: MemOff) -> Self::Output {
            PhyMemAddr(self.0 + other.0)
        }
    }
}
