use alloc::vec::Vec;

#[cfg(any(feature = "tcp", feature = "fs"))]
use crate::lib::synch::spinlock::SpinlockIrqSave;
use crate::util::irqsave;

#[cfg(any(feature = "tcp", feature = "fs"))]
use crate::drivers::virtio::transport::mmio::{
    init_device, DevId, MmioRegisterLayout, VirtioDriver,
};

#[cfg(feature = "tcp")]
use crate::board::{VIRTIO_NET_MMIO_START, VIRTIO_NET_MMIO_END, VIRTIO_NET_IRQ_NUMBER};
#[cfg(feature = "tcp")]
use crate::drivers::net::virtio_net::VirtioNetDriver;
#[cfg(feature = "tcp")]
use crate::drivers::net::NetworkInterface;

#[cfg(feature = "fs")]
use crate::board::{VIRTIO_BLK_MMIO_START, VIRTIO_BLK_MMIO_END, VIRTIO_BLK_IRQ_NUMBER};
#[cfg(feature = "fs")]
use crate::drivers::blk::virtio_blk::VirtioBlkDriver;
#[cfg(feature = "fs")]
use crate::drivers::blk::BlkInterface;

pub const MAGIC_VALUE: u32 = 0x74726976;

static mut MMIO_DRIVERS: Vec<MmioDriver> = Vec::new();

pub enum MmioDriver {
    #[cfg(feature = "tcp")]
    VirtioNet(SpinlockIrqSave<VirtioNetDriver>),
    #[cfg(feature = "fs")]
    VirtioBlk(SpinlockIrqSave<VirtioBlkDriver>),
}

impl MmioDriver {
    #[cfg(feature = "tcp")]
    #[allow(unreachable_patterns)]
    fn get_network_driver(&self) -> Option<&SpinlockIrqSave<dyn NetworkInterface>> {
        match self {
            Self::VirtioNet(drv) => Some(drv),
            _ => None,
        }
    }
    #[cfg(feature = "fs")]
    #[allow(unreachable_patterns)]
    fn get_blk_driver(&self) -> Option<&SpinlockIrqSave<dyn BlkInterface>> {
        match self {
            Self::VirtioBlk(drv) => Some(drv),
            _ => None,
        }
    }
}

#[cfg(feature = "tcp")]
/// Tries to find the network device within the specified address range.
/// Returns a reference to it within the Ok() if successful or an Err() on failure.
pub fn detect_network() -> Result<&'static mut MmioRegisterLayout, &'static str> {
    // Look for the device-ID in all possible 64-byte aligned addresses within this range.
    for current_address in (VIRTIO_NET_MMIO_START..VIRTIO_NET_MMIO_END).step_by(512) {
        debug!(
            "try to detect Virtio Network MMIO device at physical address {:#X}",
            current_address
        );

        // Verify the first register value to find out if this is really an MMIO magic-value.
        let mmio = unsafe { &mut *(current_address as *mut MmioRegisterLayout) };

        let magic = mmio.get_magic_value();
        let version = mmio.get_version();

        if magic != MAGIC_VALUE {
            debug!("It's not a MMIO-device at {:#X}", mmio as *const _ as usize);
            continue;
        }

        if version != 2 {
            debug!("Found a legacy device, which isn't supported");
            continue;
        }

        // We found a MMIO-device (whose 512-bit address in this structure).
        debug!("Found a MMIO-device at {:#X}", mmio as *const _ as usize);

        // Verify the device-ID to find the network card
        let id = mmio.get_device_id();

        if id != DevId::VIRTIO_DEV_ID_NET {
            debug!(
                "It's not a network card at {:#X}",
                mmio as *const _ as usize
            );
            continue;
        }

        info!("Found network card at {:#X}", mmio as *const _ as usize);

        // mmio.print_information();

        return Ok(mmio);
    }

    Err("Network card not found!")
}

#[cfg(feature = "fs")]
pub fn detect_blk() -> Result<&'static mut MmioRegisterLayout, &'static str> {
    // Look for the device-ID in all possible 64-byte aligned addresses within this range.
    for current_address in (VIRTIO_BLK_MMIO_START..VIRTIO_BLK_MMIO_END).step_by(512) {
        debug!(
            "try to detect Virtio Block MMIO device at physical address {:#X}",
            current_address
        );

        // Verify the first register value to find out if this is really an MMIO magic-value.
        let mmio = unsafe { &mut *(current_address as *mut MmioRegisterLayout) };

        let magic = mmio.get_magic_value();
        let version = mmio.get_version();

        if magic != MAGIC_VALUE {
            debug!("It's not a MMIO-device at {:#X}", mmio as *const _ as usize);
            continue;
        }

        if version != 2 {
            debug!("Found a legacy device, which isn't supported");
            continue;
        }

        // We found a MMIO-device (whose 512-bit address in this structure).
        debug!("Found a MMIO-device at {:#X}", mmio as *const _ as usize);

        // Verify the device-ID to find the network card
        let id = mmio.get_device_id();

        if id != DevId::VIRTIO_DEV_ID_BLK {
            debug!(
                "It's not a blk device at {:#X}",
                mmio as *const _ as usize
            );
            continue;
        }

        info!("Found blk device at {:#X}", mmio as *const _ as usize);

        // mmio.print_information();

        return Ok(mmio);
    }

    Err("Blk device not found!")
}

#[cfg(feature = "tcp")]
pub fn get_network_driver() -> Option<&'static SpinlockIrqSave<dyn NetworkInterface>> {
    unsafe { MMIO_DRIVERS.iter().find_map(|drv| drv.get_network_driver()) }
}

#[cfg(feature = "fs")]
pub fn get_block_driver() -> Option<&'static SpinlockIrqSave<dyn BlkInterface>> {
    unsafe { MMIO_DRIVERS.iter().find_map(|drv| drv.get_blk_driver()) }
}

pub fn register_driver(drv: MmioDriver) {
    unsafe {
        MMIO_DRIVERS.push(drv);
    }
}

pub fn init_drivers() {
    // virtio: MMIO Device Discovery
    irqsave(|| {
        #[cfg(feature = "tcp")]
        if let Ok(mmio) = detect_network() {
            debug!(
                "Found MMIO device, but we guess the interrupt number {}!",
                VIRTIO_NET_IRQ_NUMBER
            );
            if let Ok(VirtioDriver::Network(drv)) = init_device(mmio, VIRTIO_NET_IRQ_NUMBER) {
                register_driver(MmioDriver::VirtioNet(SpinlockIrqSave::new(drv)))
            }
        } else {
            debug!("Unable to find network mmio device");
        }
        #[cfg(feature = "fs")]
        if let Ok(mmio) = detect_blk() {
            debug!(
                "Found MMIO device, but we guess the interrupt number {}!",
                VIRTIO_BLK_IRQ_NUMBER
            );
            if let Ok(VirtioDriver::Blk(drv)) = init_device(mmio, VIRTIO_BLK_IRQ_NUMBER) {
                register_driver(MmioDriver::VirtioBlk(SpinlockIrqSave::new(drv)))
            }
        } else {
            debug!("Unable to find network mmio device");
        }
        #[cfg(feature = "oldfs")]
        crate::drivers::blk::virtio_blk_init();
    });
}
