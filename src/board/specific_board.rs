/// Dummy configure for specific board configs.

pub const BOARD_CORE_NUMBER: usize = 1;

// pub const GLOBAL_HEAP_SIZE: usize = 64 * 1024 * 1024; // 64 MB
pub const GLOBAL_HEAP_SIZE: usize = 16 * 1024 * 1024; // 16 MB

pub const ELF_IMAGE_LOAD_ADDR: usize = 0xdeadbeef;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "axdriver", feature = "pci"))] {
        // Base physical address of the PCIe ECAM space (should read from ACPI 'MCFG' table).
        pub const PCI_ECAM_BASE: usize = 0xdeadbeef;
        /// End PCI bus number.
        pub const PCI_BUS_END: usize = 0x1;
        /// PCI device memory ranges.
        pub const PCI_RANGES: &[(usize, usize)] = &[];
    }
}

pub fn init() {}

pub fn init_per_core() {}

#[cfg(feature = "smp")]
pub fn launch_other_cores() {
    warn!("Unimplented!!! starting to launch other cores...");
}
