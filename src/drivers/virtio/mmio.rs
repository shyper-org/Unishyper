use crate::libs::device::Device;
use crate::libs::traits::Address;

use alloc::vec::Vec;
use core::ops::Range;

#[cfg(any(feature = "net", feature = "fat"))]
use crate::libs::synch::spinlock::SpinlockIrqSave;
use crate::util::irqsave;

#[cfg(any(feature = "net", feature = "fat"))]
use crate::drivers::virtio::transport::mmio::{init_device, DevId, MmioRegisterLayout, VirtioDriver};

#[cfg(feature = "net")]
use crate::drivers::net::virtio_net::VirtioNetDriver;
#[cfg(feature = "net")]
use crate::drivers::net::NetworkInterface;

#[cfg(feature = "fat")]
use crate::drivers::blk::virtio_blk::VirtioBlkDriver;
#[cfg(feature = "fat")]
use crate::drivers::blk::BlkInterface;

pub const MAGIC_VALUE: u32 = 0x74726976;

static mut MMIO_DRIVERS: Vec<MmioDriver> = Vec::new();

pub enum MmioDriver {
    #[cfg(feature = "net")]
    VirtioNet(SpinlockIrqSave<VirtioNetDriver>),
    #[cfg(feature = "fat")]
    VirtioBlk(SpinlockIrqSave<VirtioBlkDriver>),
}

impl MmioDriver {
    #[cfg(feature = "net")]
    #[allow(unreachable_patterns)]
    fn get_network_driver(&self) -> Option<&SpinlockIrqSave<dyn NetworkInterface>> {
        match self {
            Self::VirtioNet(drv) => Some(drv),
            _ => None,
        }
    }
    #[cfg(feature = "fat")]
    #[allow(unreachable_patterns)]
    fn get_blk_driver(&self) -> Option<&SpinlockIrqSave<dyn BlkInterface>> {
        match self {
            Self::VirtioBlk(drv) => Some(drv),
            _ => None,
        }
    }
}

#[cfg(any(feature = "net", feature = "fat"))]
fn init_virtio_device(
    range: Range<usize>,
) -> Result<&'static mut MmioRegisterLayout, &'static str> {
    // Verify the first register value to find out if this is really an MMIO magic-value.
    debug!("init_virtio_device @ 0x{:x}", range.start.pa2kva());
    let mmio = unsafe { &mut *(range.start.pa2kva() as *mut MmioRegisterLayout) };

    if mmio.get_magic_value() != MAGIC_VALUE {
        return Err("It's not a MMIO-device");
    }

    if mmio.get_version() != 2 {
        return Err("Found a legacy device, which isn't supported");
    }

    match mmio.get_device_id() {
        DevId::VIRTIO_DEV_ID_NET => {
            debug!("Found Net device type {}", u32::from(mmio.get_device_id()));
            Ok(mmio)
        }
        DevId::VIRTIO_DEV_ID_BLK => {
            debug!(
                "Found Block device type {}",
                u32::from(mmio.get_device_id())
            );
            Ok(mmio)
        }
        _ => Err("INVALID device"),
    }
}

#[cfg(feature = "net")]
pub fn get_network_driver() -> Option<&'static SpinlockIrqSave<dyn NetworkInterface>> {
    unsafe { MMIO_DRIVERS.iter().find_map(|drv| drv.get_network_driver()) }
}

#[cfg(feature = "fat")]
pub fn get_block_driver() -> Option<&'static SpinlockIrqSave<dyn BlkInterface>> {
    unsafe { MMIO_DRIVERS.iter().find_map(|drv| drv.get_blk_driver()) }
}

#[cfg(any(feature = "net", feature = "fat"))]
fn parse_virtio_devices() {
    let devices = crate::board::devices();
    for device in devices {
        match device {
            Device::Virtio(device) => match init_virtio_device(device.registers) {
                Ok(mmio) => {
                    let driver = match init_device(mmio, device.interrupts as u32) {
                        #[cfg(feature = "fat")]
                        Ok(VirtioDriver::Blk(drv)) => {
                            MmioDriver::VirtioBlk(SpinlockIrqSave::new(drv))
                        }
                        #[cfg(feature = "net")]
                        Ok(VirtioDriver::Network(drv)) => {
                            MmioDriver::VirtioNet(SpinlockIrqSave::new(drv))
                        }
                        Err(_) => panic!("init device Error"),
                    };
                    info!("Virtio device [\"{}\'] init ok!", device.name);
                    register_driver(driver);
                }
                Err(e) => {
                    warn!("Device {:?} init failed for {:?}", device.name, e);
                }
            },
            Device::Unknown => panic!("Unsupported Device"),
        }
    }
}

pub fn register_driver(drv: MmioDriver) {
    unsafe {
        MMIO_DRIVERS.push(drv);
    }
}

pub fn init_drivers() {
    // virtio: MMIO Device Discovery
    irqsave(|| {
        #[cfg(any(feature = "net", feature = "fat"))]
        parse_virtio_devices();
    });
}
