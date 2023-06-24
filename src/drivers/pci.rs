#![allow(dead_code)]

use alloc::vec::Vec;
use core::{fmt, u32, u8};

use x86_64::instructions::port::{PortWriteOnly, PortReadOnly};
use crate::libs::synch::spinlock::SpinlockIrqSave;
use crate::util::irqsave;
// use x86::io::*;

// use crate::mm::{PhysAddr, VirtAddr};
use crate::drivers::net::virtio_net::VirtioNetDriver;
use crate::drivers::net::NetworkInterface;
use crate::drivers::virtio::transport::pci as pci_virtio;
use crate::drivers::virtio::transport::pci::VirtioDriver;

// TODO: should these be pub? currently needed since used in virtio.rs maybe use getter methods to be more flexible.
pub const PCI_MAX_BUS_NUMBER: u8 = 32;
pub const PCI_MAX_DEVICE_NUMBER: u8 = 32;

pub const PCI_CONFIG_ADDRESS_PORT: u16 = 0xCF8;
pub const PCI_CONFIG_ADDRESS_ENABLE: u32 = 1 << 31;

pub const PCI_CONFIG_DATA_PORT: u16 = 0xCFC;
pub const PCI_COMMAND_BUSMASTER: u32 = 1 << 2;

pub const PCI_ID_REGISTER: u32 = 0x00;
pub const PCI_COMMAND_REGISTER: u32 = 0x04;
pub const PCI_CLASS_REGISTER: u32 = 0x08;
pub const PCI_HEADER_REGISTER: u32 = 0x0C;
pub const PCI_BAR0_REGISTER: u32 = 0x10;
pub const PCI_CAPABILITY_LIST_REGISTER: u32 = 0x34;
pub const PCI_INTERRUPT_REGISTER: u32 = 0x3C;

pub const PCI_STATUS_CAPABILITIES_LIST: u32 = 1 << 4;

pub const PCI_BASE_ADDRESS_IO_SPACE: u32 = 1 << 0;
pub const PCI_MEM_BASE_ADDRESS_64BIT: u32 = 1 << 2;
pub const PCI_MEM_PREFETCHABLE: u32 = 1 << 3;
pub const PCI_MEM_BASE_ADDRESS_MASK: u32 = 0xFFFF_FFF0;
pub const PCI_IO_BASE_ADDRESS_MASK: u32 = 0xFFFF_FFFC;

pub const PCI_HEADER_TYPE_MASK: u32 = 0x007F_0000;
pub const PCI_MULTIFUNCTION_MASK: u32 = 0x0080_0000;

pub const PCI_CAP_ID_VNDR: u32 = 0x09;

static mut PCI_ADAPTERS: Vec<PciAdapter> = Vec::new();
static mut PCI_DRIVERS: Vec<PciDriver> = Vec::new();

#[derive(Clone, Debug)]
pub struct PciAdapter {
    pub bus: u8,
    pub device: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_id: u8,
    pub subclass_id: u8,
    pub programming_interface_id: u8,
    pub base_addresses: Vec<PciBar>,
    pub irq: u8,
}
#[derive(Clone, Copy, Debug)]
pub enum PciBar {
    IO(IOBar),
    Memory(MemoryBar),
}
#[derive(Clone, Copy, Debug)]
pub struct IOBar {
    pub index: u8,
    pub addr: u32,
    pub size: usize,
}
#[derive(Clone, Copy, Debug)]
pub struct MemoryBar {
    pub index: u8,
    pub addr: usize,
    pub size: usize,
    pub width: u8, // 32 or 64 bit
    pub prefetchable: bool,
}

#[allow(clippy::large_enum_variant)]
pub enum PciDriver {
    // VirtioFs(SpinlockIrqSave<VirtioFsDriver>),
    VirtioNet(SpinlockIrqSave<VirtioNetDriver>),
}

impl PciDriver {
    fn get_network_driver(&self) -> Option<&SpinlockIrqSave<dyn NetworkInterface>> {
        match self {
            Self::VirtioNet(drv) => Some(drv),
            // _ => None,
        }
    }
}

pub fn register_driver(drv: PciDriver) {
    unsafe {
        PCI_DRIVERS.push(drv);
    }
}

pub fn get_network_driver() -> Option<&'static SpinlockIrqSave<dyn NetworkInterface>> {
    unsafe { PCI_DRIVERS.iter().find_map(|drv| drv.get_network_driver()) }
}

/// Reads all bar registers of specified device and returns vector of PciBar's containing addresses and sizes.
fn parse_bars(bus: u8, device: u8, vendor_id: u16, device_id: u16) -> Vec<PciBar> {
    let mut bar_idxs = 0..6;
    let mut bars = Vec::new();
    while let Some(i) = bar_idxs.next() {
        let register = PCI_BAR0_REGISTER + (i << 2);
        let barword = read_config(bus, device, register);
        debug!(
            "Found bar{} @{:x}:{:x} as {:#x}",
            i,
            vendor_id,
            device_id,
            barword
        );

        // We assume BIOS or something similar has initialized the device already and set appropriate values into the bar registers

        // If barword is all 0, the bar is disabled
        if barword == 0 {
            continue;
        }

        // Determine if bar is IO-mapped or memory-mapped
        if barword & PCI_BASE_ADDRESS_IO_SPACE != 0 {
            // IO Mapped BAR
            debug!("Bar {} @{:x}:{:x} IO mapped!", i, vendor_id, device_id);

            let base_addr = barword & PCI_IO_BASE_ADDRESS_MASK;

            // determine size by writing 0xFFFFFFFF
            write_config(bus, device, register, u32::MAX);
            let sizebits = read_config(bus, device, register);
            // Restore original value of register
            write_config(bus, device, register, barword);
            let size = (!(sizebits & PCI_IO_BASE_ADDRESS_MASK) + 1) as usize;

            bars.push(PciBar::IO(IOBar {
                index: i as u8,
                addr: base_addr,
                size,
            }));
        } else {
            // Memory Mapped BAR
            let prefetchable = barword & PCI_MEM_PREFETCHABLE != 0;

            if barword & PCI_MEM_BASE_ADDRESS_64BIT != 0 {
                // 64-bit, load additional bar-word
                let register_high = PCI_BAR0_REGISTER + (bar_idxs.next().unwrap() << 2);
                let barword_high = read_config(bus, device, register_high);

                let base_addr = ((barword_high as usize) << 32) + (barword & 0xFFFF_FFF0) as usize;
                debug!(
                    "64-bit memory bar, merged next barword. Addr: {:#x}",
                    base_addr
                );

                // determine size by writing 0xFFFFFFFF
                write_config(bus, device, register, u32::MAX);
                let sizebits = read_config(bus, device, register);

                // Also read/write to register_high if needed
                let size = if sizebits == 0 {
                    write_config(bus, device, register_high, u32::MAX);
                    let sizebits = read_config(bus, device, register_high);
                    // Restore original value of register_high
                    write_config(bus, device, register_high, barword);

                    ((!sizebits + 1) as usize) << 32
                } else {
                    (!(sizebits & PCI_MEM_BASE_ADDRESS_MASK) + 1) as usize
                };

                // Restore original value
                write_config(bus, device, register, barword);

                bars.push(PciBar::Memory(MemoryBar {
                    index: i as u8,
                    addr: base_addr,
                    size,
                    width: 64,
                    prefetchable,
                }));
            } else {
                // 32-bit
                let base_addr = (barword & 0xFFFF_FFF0) as usize;

                // determine size by writing 0xFFFFFFFF
                write_config(bus, device, register, u32::MAX);
                let size = !(read_config(bus, device, register) & PCI_MEM_BASE_ADDRESS_MASK) + 1;

                // Restore original value
                write_config(bus, device, register, barword);

                bars.push(PciBar::Memory(MemoryBar {
                    index: i as u8,
                    addr: base_addr,
                    size: size.try_into().unwrap(),
                    width: 32,
                    prefetchable,
                }));
            }
        }
    }

    bars
}

impl PciAdapter {
    fn new(bus: u8, device: u8, vendor_id: u16, device_id: u16) -> Option<Self> {
        let header = read_config(bus, device, PCI_HEADER_REGISTER);
        if header & PCI_HEADER_TYPE_MASK != 0 {
            error!(
                "PCI Device @{:x}:{:x} does not have header type 0!",
                vendor_id, device_id
            );
            return None;
        }
        if header & PCI_MULTIFUNCTION_MASK != 0 {
            warn!(
                "PCI Device @{:x}:{:x} has multiple functions! Currently only one is handled.",
                vendor_id, device_id
            );
        }

        let class_ids = read_config(bus, device, PCI_CLASS_REGISTER);
        let bars = parse_bars(bus, device, vendor_id, device_id);
        let interrupt_info = read_config(bus, device, PCI_INTERRUPT_REGISTER);

        Some(Self {
            bus,
            device,
            vendor_id,
            device_id,
            class_id: (class_ids >> 24) as u8,
            subclass_id: (class_ids >> 16) as u8,
            programming_interface_id: (class_ids >> 8) as u8,
            base_addresses: bars,
            irq: interrupt_info as u8,
        })
    }

    pub fn make_bus_master(&self) {
        let mut command = read_config(self.bus, self.device, PCI_COMMAND_REGISTER);
        command |= PCI_COMMAND_BUSMASTER;
        write_config(self.bus, self.device, PCI_COMMAND_REGISTER, command);
    }

    /// Returns the bar at bar-register baridx.
    pub fn get_bar(&self, baridx: u8) -> Option<PciBar> {
        for pci_bar in &self.base_addresses {
            match pci_bar {
                PciBar::IO(pci_bar) => {
                    if pci_bar.index == baridx {
                        return Some(PciBar::IO(*pci_bar));
                    }
                }
                PciBar::Memory(pci_bar) => {
                    if pci_bar.index == baridx {
                        return Some(PciBar::Memory(*pci_bar));
                    }
                }
            }
        }
        None
    }
}

impl fmt::Display for PciBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (typ, addr, size) = match self {
            PciBar::IO(io_bar) => ("IOBar", io_bar.addr as usize, io_bar.size),
            PciBar::Memory(mem_bar) => ("MemoryBar", mem_bar.addr, mem_bar.size),
        };
        write!(f, "{typ}: {addr:#x} (size {size:#x})")?;

        Ok(())
    }
}

impl fmt::Display for PciAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "pci-ids")]
        use pci_ids::{Class, Device, FromId, Subclass};

        #[cfg(feature = "pci-ids")]
        let class_name = Class::from_id(self.class_id).map_or("Unknown Class", |class| {
            class
                .subclasses()
                .find(|s| s.id() == self.subclass_id)
                .map(Subclass::name)
                .unwrap_or_else(|| class.name())
        });

        #[cfg(feature = "pci-ids")]
        let (vendor_name, device_name) = Device::from_vid_pid(self.vendor_id, self.device_id)
            .map(|device| (device.vendor().name(), device.name()))
            .unwrap_or(("Unknown Vendor", "Unknown Device"));

        #[cfg(not(feature = "pci-ids"))]
        let (class_name, vendor_name, device_name) =
            ("Unknown Class", "Unknown Vendor", "Unknown Device");

        // Output detailed readable information about this device.
        write!(
            f,
            "{:02X}:{:02X} {} [{:02X}{:02X}]: {} {} [{:04X}:{:04X}]",
            self.bus,
            self.device,
            class_name,
            self.class_id,
            self.subclass_id,
            vendor_name,
            device_name,
            self.vendor_id,
            self.device_id
        )?;

        // If the devices uses an IRQ, output this one as well.
        if self.irq != 0 && self.irq != u8::MAX {
            write!(f, ", IRQ {}", self.irq)?;
        }

        for pci_bar in &self.base_addresses {
            write!(f, ", {pci_bar}")?;
        }

        Ok(())
    }
}

/// Returns the value (indicated by bus, device and register) of the pci
/// configuration space.
pub fn read_config(bus: u8, device: u8, register: u32) -> u32 {
    let address =
        PCI_CONFIG_ADDRESS_ENABLE | u32::from(bus) << 16 | u32::from(device) << 11 | register;
    let mut config_address_port = PortWriteOnly::<u32>::new(PCI_CONFIG_ADDRESS_PORT);
    let mut config_data_port = PortReadOnly::<u32>::new(PCI_CONFIG_DATA_PORT);
    // let pci_config_address_port = PortWriteOnly::new(port)
    unsafe {
        // outl(PCI_CONFIG_ADDRESS_PORT, address);
        // inl(PCI_CONFIG_DATA_PORT)
        config_address_port.write(address);
        config_data_port.read()
    }
}

pub fn write_config(bus: u8, device: u8, register: u32, data: u32) {
    let address =
        PCI_CONFIG_ADDRESS_ENABLE | u32::from(bus) << 16 | u32::from(device) << 11 | register;
    let mut config_address_port = PortWriteOnly::<u32>::new(PCI_CONFIG_ADDRESS_PORT);
    let mut config_data_port = PortWriteOnly::<u32>::new(PCI_CONFIG_DATA_PORT);
    unsafe {
        // outl(PCI_CONFIG_ADDRESS_PORT, address);
        // outl(PCI_CONFIG_DATA_PORT, data);
        config_address_port.write(address);
        config_data_port.write(data)
    }
}

pub fn init() {
    debug!("Scanning PCI Busses 0 to {}", PCI_MAX_BUS_NUMBER - 1);

    // HermitCore only uses PCI for network devices.
    // Therefore, multifunction devices as well as additional bridges are not scanned.
    // We also limit scanning to the first 32 buses.
    for bus in 0..PCI_MAX_BUS_NUMBER {
        for device in 0..PCI_MAX_DEVICE_NUMBER {
            let device_vendor_id = read_config(bus, device, PCI_ID_REGISTER);
            if device_vendor_id != u32::MAX {
                let device_id = (device_vendor_id >> 16) as u16;
                let vendor_id = device_vendor_id as u16;
                let adapter = PciAdapter::new(bus, device, vendor_id, device_id);
                if let Some(adapter) = adapter {
                    unsafe {
                        PCI_ADAPTERS.push(adapter);
                    }
                }
            }
        }
    }
}

pub fn init_drivers() {
    let mut nic_available = false;
    debug!("init pci drivers");
    // virtio: 4.1.2 PCI Device Discovery
    irqsave(|| {
        for adapter in unsafe {
            PCI_ADAPTERS
                .iter()
                .filter(|x| x.vendor_id == 0x1AF4 && x.device_id >= 0x1000 && x.device_id <= 0x107F)
        } {
            info!(
                "Found virtio network device with device id {:#x}",
                adapter.device_id
            );

            match pci_virtio::init_device(adapter) {
                Ok(VirtioDriver::Network(drv)) => {
                    nic_available = true;
                    register_driver(PciDriver::VirtioNet(SpinlockIrqSave::new(drv)))
                }
                Err(err) => {
                    warn!("PCI device init error {}", err);
                }
            }
        }
    });
}

pub fn print_information() {
    infoheader!(" PCI BUS INFORMATION ");

    for adapter in unsafe { PCI_ADAPTERS.iter() } {
        info!("{}", adapter);
    }

    infofooter!();
}

/// A module containing PCI specific errors
///
/// Errors include...
pub mod error {
    /// An enum of PciErrors
    /// typically carrying the device's id as an u16.
    #[derive(Debug)]
    pub enum PciError {
        General(u16),
        NoBar(u16),
        NoCapPtr(u16),
        BadCapPtr(u16),
        NoVirtioCaps(u16),
    }
}
