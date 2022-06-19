use alloc::vec::Vec;

use crate::drivers::net::virtio_net::VirtioNetDriver;
use crate::drivers::net::NetworkInterface;
use crate::drivers::virtio::transport::mmio::{
    init_device, DevId, MmioRegisterLayout, VirtioDriver,
};
use crate::lib::synch::spinlock::SpinlockIrqSave;
use crate::util::irqsave;

pub const MAGIC_VALUE: u32 = 0x74726976;

pub const VIRTIO_MMIO_START: usize = 0xFFFF_FF80_0000_0000 + 0x0a00_0000;
pub const VIRTIO_MMIO_END: usize = 0xFFFF_FF80_0000_0000 + 0x0a01_0000;
const IRQ_NUMBER: u32 = 12;

static mut MMIO_DRIVERS: Vec<MmioDriver> = Vec::new();

pub enum MmioDriver {
    VirtioNet(SpinlockIrqSave<VirtioNetDriver>),
}

impl MmioDriver {
    #[allow(unreachable_patterns)]
    fn get_network_driver(&self) -> Option<&SpinlockIrqSave<dyn NetworkInterface>> {
        match self {
            Self::VirtioNet(drv) => Some(drv),
            _ => None,
        }
    }
}

/// Tries to find the network device within the specified address range.
/// Returns a reference to it within the Ok() if successful or an Err() on failure.
pub fn detect_network() -> Result<&'static mut MmioRegisterLayout, &'static str> {
    // Look for the device-ID in all possible 64-byte aligned addresses within this range.
    for current_address in (VIRTIO_MMIO_START..VIRTIO_MMIO_END).step_by(512) {
        trace!(
            "try to detect MMIO device at physical address {:#X}",
            current_address
        );
        debug!(
            "try to detect MMIO device at physical address {:#X}",
            current_address
        );

        // Verify the first register value to find out if this is really an MMIO magic-value.
        let mmio = unsafe { &mut *(current_address as *mut MmioRegisterLayout) };

        let magic = mmio.get_magic_value();
        let version = mmio.get_version();

        if magic != MAGIC_VALUE {
            trace!("It's not a MMIO-device at {:#X}", mmio as *const _ as usize);
            continue;
        }

        if version != 2 {
            trace!("Found a legacy device, which isn't supported");
            continue;
        }

        // We found a MMIO-device (whose 512-bit address in this structure).
        trace!("Found a MMIO-device at {:#X}", mmio as *const _ as usize);

        // Verify the device-ID to find the network card
        let id = mmio.get_device_id();

        if id != DevId::VIRTIO_DEV_ID_NET {
            trace!(
                "It's not a network card at {:#X}",
                mmio as *const _ as usize
            );
            continue;
        }

        info!("Found network card at {:#X}", mmio as *const _ as usize);

        mmio.print_information();

        return Ok(mmio);
    }

    Err("Network card not found!")
}

pub fn get_network_driver() -> Option<&'static SpinlockIrqSave<dyn NetworkInterface>> {
    unsafe { MMIO_DRIVERS.iter().find_map(|drv| drv.get_network_driver()) }
}

pub fn register_driver(drv: MmioDriver) {
    unsafe {
        MMIO_DRIVERS.push(drv);
    }
}

pub fn init_drivers() {
    // virtio: MMIO Device Discovery
    irqsave(|| {
        if let Ok(mmio) = detect_network() {
            debug!(
                "Found MMIO device, but we guess the interrupt number {}!",
                IRQ_NUMBER
            );
            if let Ok(VirtioDriver::Network(drv)) = init_device(mmio, IRQ_NUMBER) {
                register_driver(MmioDriver::VirtioNet(SpinlockIrqSave::new(drv)))
            }
        } else {
            debug!("Unable to find mmio device");
        }
    });
}
