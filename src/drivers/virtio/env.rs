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

/// This module is used as a single entry point from Virtio code into
/// other parts of the kernel.
///
/// INFO: Values passed on to PCI devices are automatically converted into little endian
/// coding. Values provided from PCI devices are passed as native endian values.
/// Meaning they are converted into big endian values on big endian machines and
/// are not changed on little endian machines.
#[cfg(feature = "pci")]
pub mod pci {
    use alloc::vec::Vec;
    use core::result::Result;

    use crate::drivers::pci;
    use crate::drivers::pci::error::PciError;
    use crate::drivers::pci::{PciAdapter, PciBar};
    use crate::drivers::virtio::env::memory::VirtMemAddr;
    use crate::drivers::virtio::transport::pci::PciBar as VirtioPciBar;

    /// Wrapper function to read the configuration space of a PCI
    /// device at the given register. Returns the registers value.
    pub fn read_config(adapter: &PciAdapter, register: u32) -> u32 {
        from_pci_endian(pci::read_config(
            adapter.bus,
            adapter.device,
            register.to_le(),
        ))
    }

    /// Wrapper function to read the configuration space of a PCI
    /// device at the given register. Returns the registers value.
    pub fn read_cfg_no_adapter(bus: u8, device: u8, register: u32) -> u32 {
        from_pci_endian(pci::read_config(bus, device, register.to_le()))
    }

    /// Wrapper function to write the configuration space of a PCI
    /// device at the given register.
    #[allow(dead_code)]
    pub fn write_config(adapter: &PciAdapter, register: u32, data: u32) {
        pci::write_config(adapter.bus, adapter.device, register.to_le(), data.to_le());
    }

    /// Converts a given little endian coded u32 to native endian coded.
    //
    // INFO: As the endianness received from the device is little endian coded
    // the given value must be swapped again on big endian machines. Which is done
    // via the u32::to_le() method as the u32::to_be() would be a no-op in big endian
    // machines. Resulting in no conversion.
    fn from_pci_endian(val: u32) -> u32 {
        if cfg!(target = "big_endian") {
            val.to_le()
        } else {
            val
        }
    }

    /// Maps all memory areas indicated by the devices BAR's into
    /// Virtual address space.
    ///
    /// As this function uses parts of the kernel pci code it is
    /// outsourced into the env::pci module.
    ///
    /// WARN: Currently unsafely casts kernel::PciBar.size (usize) to an
    /// u64
    pub fn map_bar_mem(adapter: &PciAdapter) -> Result<Vec<VirtioPciBar>, PciError> {
        let mut mapped_bars: Vec<VirtioPciBar> = Vec::new();

        for bar in &adapter.base_addresses {
            match bar {
                PciBar::IO(_) => {
                    warn!("Cannot map I/O BAR!");
                    continue;
                }
                PciBar::Memory(bar) => {
                    debug!("pci mem bar {:?}", bar);
                    if bar.width != 64 {
                        warn!("Currently only mapping of 64 bit BAR's is supported!");
                        continue;
                    }
                    if !bar.prefetchable {
                        warn!("Currently only mapping of prefetchable BAR's is supported!");
                        continue;
                    }

                    debug!("bar addr {:#x} size {:#x}", bar.addr, bar.size);

                    let virtual_address = VirtMemAddr::from(
                        // crate::mm::map(PhysAddr::from(bar.addr), bar.size, true, true, true).0,
                        // bar.addr.pa2kva()
                        crate::mm::paging::map_pci_mem_bar(bar.addr, bar.size)
                    );

                    mapped_bars.push(VirtioPciBar::new(
                        bar.index,
                        virtual_address,
                        bar.size as u64,
                    ));
                }
            }
        }

        if mapped_bars.is_empty() {
            error!(
                "No correct memory BAR for device {:x} found.",
                adapter.device_id
            );
            Err(PciError::NoBar(adapter.device_id))
        } else {
            Ok(mapped_bars)
        }
    }
}
