//! Defines types and probe methods of all supported devices.

#![allow(unused_imports)]

use super::AxDeviceEnum;
use driver_common::DeviceType;

use crate::drivers::axdriver::virtio::{self, VirtIoDevMeta};

#[cfg(feature = "pci")]
use driver_pci::{DeviceFunction, DeviceFunctionInfo, PciRoot};

pub use super::dummy::*;

pub trait DriverProbe {
    fn probe_global() -> Option<AxDeviceEnum> {
        None
    }

    #[cfg(feature = "mmio")]
    fn probe_mmio(_mmio_base: usize, _mmio_size: usize) -> Option<AxDeviceEnum> {
        None
    }

    #[cfg(feature = "pci")]
    fn probe_pci(
        _root: &mut PciRoot,
        _bdf: DeviceFunction,
        _dev_info: &DeviceFunctionInfo,
    ) -> Option<AxDeviceEnum> {
        None
    }
}

// pub type AxNetDevice =  <virtio::VirtIoNet as VirtIoDevMeta>::Device;
