//! A module containing all virtio specific pci functionality
//!
//! The module contains ...
#![allow(dead_code)]

use core::convert::TryInto;
use core::result::Result;
use core::sync::atomic::fence;
use core::sync::atomic::Ordering;
use core::u8;

use tock_registers::*;
use tock_registers::interfaces::*;
use tock_registers::registers::*;

use crate::drivers::error::DriverError;
use crate::drivers::virtio::device;
use crate::drivers::virtio::error::VirtioError;

use crate::lib::interrupt::irq_install_handler;

#[cfg(feature = "tcp")]
use crate::drivers::net::virtio_net::VirtioNetDriver;
#[cfg(feature = "tcp")]
use crate::drivers::net::network_irqhandler;

#[cfg(feature = "fs")]
use crate::drivers::blk::virtio_blk::VirtioBlkDriver;
#[cfg(feature = "fs")]
use crate::drivers::blk::blk_irqhandler;
// use crate::drivers::blk::;

/// Virtio device ID's
/// See Virtio specification v1.1. - 5
///
// WARN: Upon changes in the set of the enum variants
// one MUST adjust the associated From<u32>
// implementation, in order catch all cases correctly,
// as this function uses the catch-all "_" case!
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u32)]
pub enum DevId {
    INVALID = 0x0,
    VIRTIO_DEV_ID_NET = 1,
    VIRTIO_DEV_ID_BLK = 2,
    VIRTIO_DEV_ID_CONSOLE = 3,
}

impl From<DevId> for u32 {
    fn from(val: DevId) -> u32 {
        match val {
            DevId::VIRTIO_DEV_ID_NET => 1,
            DevId::VIRTIO_DEV_ID_BLK => 2,
            DevId::VIRTIO_DEV_ID_CONSOLE => 3,
            DevId::INVALID => 0x0,
        }
    }
}

impl From<u32> for DevId {
    fn from(val: u32) -> Self {
        match val {
            1 => DevId::VIRTIO_DEV_ID_NET,
            2 => DevId::VIRTIO_DEV_ID_BLK,
            3 => DevId::VIRTIO_DEV_ID_CONSOLE,
            _ => DevId::INVALID,
        }
    }
}

pub struct VqCfgHandler<'a> {
    vq_index: u32,
    raw: &'a MmioRegisterLayout,
}

impl<'a> VqCfgHandler<'a> {
    /// Sets the size of a given virtqueue. In case the provided size exceeds the maximum allowed
    /// size, the size is set to this maximum instead. Else size is set to the provided value.
    ///
    /// Returns the set size in form of a `u16`.
    pub fn set_vq_size(&mut self, size: u16) -> u16 {
        self.raw
            .set_queue_size(self.vq_index, size as u32)
            .try_into()
            .unwrap()
    }

    pub fn set_ring_addr(&mut self, addr: usize) {
        self.raw.set_ring_addr(self.vq_index, addr);
    }

    pub fn set_drv_ctrl_addr(&mut self, addr: usize) {
        self.raw.set_drv_ctrl_addr(self.vq_index, addr);
    }

    pub fn set_dev_ctrl_addr(&mut self, addr: usize) {
        self.raw.set_dev_ctrl_addr(self.vq_index, addr);
    }

    pub fn notif_off(&mut self) -> u16 {
        // we don't need an offset
        0
    }

    pub fn enable_queue(&mut self) {
        self.raw.enable_queue(self.vq_index);
    }
}

/// Wraps a [ComCfgRaw](structs.comcfgraw.html) in order to preserve
/// the original structure.
///
/// Provides a safe API for Raw structure and allows interaction with the device via
/// the structure.
pub struct ComCfg {
    // References the raw structure in PCI memory space. Is static as
    // long as the device is present, which is mandatory in order to let this code work.
    com_cfg: &'static MmioRegisterLayout,

    /// Preferences of the device for this config. From 1 (highest) to 2^7-1 (lowest)
    rank: u8,
}

// Public Interface of ComCfg
impl ComCfg {
    pub fn new(raw: &'static MmioRegisterLayout, rank: u8) -> Self {
        ComCfg { com_cfg: raw, rank }
    }

    /// Select a queue via an index. If queue does NOT exist returns `None`, else
    /// returns `Some(VqCfgHandler)`.
    ///
    /// INFO: The queue size is automatically bounded by constant `src::config:VIRTIO_MAX_QUEUE_SIZE`.
    pub fn select_vq(&self, index: u16) -> Option<VqCfgHandler<'_>> {
        if self.com_cfg.get_max_queue_size(u32::from(index)) == 0 {
            None
        } else {
            Some(VqCfgHandler {
                vq_index: index as u32,
                raw: self.com_cfg,
            })
        }
    }

    pub fn get_max_queue_size(&self, sel: u32) -> u32 {
        self.com_cfg.get_max_queue_size(sel)
    }

    pub fn get_queue_ready(&self, sel: u32) -> bool {
        self.com_cfg.get_queue_ready(sel)
    }

    /// Returns the device status field.
    pub fn dev_status(&self) -> u8 {
        self.com_cfg.status.get().try_into().unwrap()
    }

    /// Resets the device status field to zero.
    pub fn reset_dev(&self) {
        self.com_cfg.status.set(0u32);
    }

    /// Sets the device status field to FAILED.
    /// A driver MUST NOT initialize and use the device any further after this.
    /// A driver MAY use the device again after a proper reset of the device.
    pub fn set_failed(&self) {
        self.com_cfg.status.set(u32::from(device::Status::FAILED));
    }

    /// Sets the ACKNOWLEDGE bit in the device status field. This indicates, the
    /// OS has notived the device
    pub fn ack_dev(&self) {
        let status = self.com_cfg.status.get();
        self.com_cfg
            .status
            .set(status | u32::from(device::Status::ACKNOWLEDGE));
    }

    /// Sets the DRIVER bit in the device status field. This indicates, the OS
    /// know how to run this device.
    pub fn set_drv(&self) {
        let status = self.com_cfg.status.get();
        self.com_cfg
            .status
            .set(status | u32::from(device::Status::DRIVER));
    }

    /// Sets the FEATURES_OK bit in the device status field.
    ///
    /// Drivers MUST NOT accept new features after this step.
    pub fn features_ok(&self) {
        let status = self.com_cfg.status.get();
        self.com_cfg
            .status
            .set(status | u32::from(device::Status::FEATURES_OK));
    }

    /// In order to correctly check feature negotiaten, this function
    /// MUST be called after [self.features_ok()](ComCfg::features_ok()) in order to check
    /// if features have been accepted by the device after negotiation.
    ///
    /// Re-reads device status to ensure the FEATURES_OK bit is still set:
    /// otherwise, the device does not support our subset of features and the device is unusable.
    pub fn check_features(&self) -> bool {
        let status = self.com_cfg.status.get();
        status & u32::from(device::Status::FEATURES_OK) == u32::from(device::Status::FEATURES_OK)
    }

    /// Sets the DRIVER_OK bit in the device status field.
    ///
    /// After this call, the device is "live"!
    pub fn drv_ok(&self) {
        let status = self.com_cfg.status.get();
        self.com_cfg
            .status
            .set(status | u32::from(device::Status::DRIVER_OK));
    }

    /// Returns the features offered by the device. Coded in a 64bit value.
    pub fn dev_features(&self) -> u64 {
        self.com_cfg.dev_features()
    }

    /// Write selected features into driver_select field.
    pub fn set_drv_features(&self, feats: u64) {
        self.com_cfg.set_drv_features(feats);
    }

    pub fn print_information(&self) {
        self.com_cfg.print_information();
    }
}

/// Notification Structure to handle virtqueue notification settings.
/// See Virtio specification v1.1 - 4.1.4.4
pub struct NotifCfg {
    /// Start addr, from where the notification addresses for the virtqueues are computed
    queue_notify: *mut u32,
}

impl NotifCfg {
    pub fn new(registers: &MmioRegisterLayout) -> Self {
        let raw = &registers.queue_notify as *const _ as *mut u32;

        NotifCfg { queue_notify: raw }
    }

    /// Returns base address of notification area as an usize
    pub fn base(&self) -> usize {
        self.queue_notify as usize
    }

    /// Returns the multiplier, needed in order to calculate the
    /// notification address for a specific queue.
    pub fn multiplier(&self) -> u32 {
        // we don't need a multiplier
        0
    }
}

/// Control structure, allowing to notify a device via PCI bus.
/// Typically hold by a virtqueue.
pub struct NotifCtrl {
    /// Indicates if VIRTIO_F_NOTIFICATION_DATA has been negotiated
    f_notif_data: bool,
    /// Where to write notification
    notif_addr: *mut u32,
}

impl NotifCtrl {
    /// Returns a new controller. By default MSI-X capabilities and VIRTIO_F_NOTIFICATION_DATA
    /// are disabled.
    pub fn new(notif_addr: *mut usize) -> Self {
        NotifCtrl {
            f_notif_data: false,
            notif_addr: notif_addr as *mut u32,
        }
    }

    /// Enables VIRTIO_F_NOTIFICATION_DATA. This changes which data is provided to the device. ONLY a good idea if Feature has been negotiated.
    pub fn enable_notif_data(&mut self) {
        self.f_notif_data = true;
    }

    pub fn notify_dev(&self, notif_data: &[u8]) {
        let data = u32::from_ne_bytes(notif_data.try_into().unwrap());
        unsafe {
            *self.notif_addr = data;
        }
    }
}

/// Wraps a [IsrStatusRaw](structs.isrstatusraw.html) in order to preserve
/// the original structure and allow interaction with the device via
/// the structure.
///
/// Provides a safe API for Raw structure and allows interaction with the device via
/// the structure.
pub struct IsrStatus {
    raw: &'static MmioRegisterLayout,
}

impl IsrStatus {
    pub fn new(registers: &'static MmioRegisterLayout) -> Self {
        IsrStatus { raw: registers }
    }

    pub fn is_interrupt(&self) -> bool {
        let status = self.raw.interrupt_status.get();
        status & 0x1 == 0x1
    }

    pub fn is_cfg_change(&self) -> bool {
        let status = self.raw.interrupt_status.get();
        status & 0x2 == 0x2
    }

    pub fn acknowledge(&self) {
        let status = self.raw.interrupt_status.get();
        self.raw.interrupt_ack.set(status);
    }
}

pub enum VirtioDriver {
    #[cfg(feature = "tcp")]
    Network(VirtioNetDriver),
    #[cfg(feature = "fs")]
    Blk(VirtioBlkDriver),
}

pub fn init_device(
    registers: &'static mut MmioRegisterLayout,
    irq_no: u32,
) -> Result<VirtioDriver, DriverError> {
    let dev_id: u16 = 0;

    if registers.get_version() == 0x1 {
        error!("Legacy interface isn't supported!");
        return Err(DriverError::InitVirtioDevFail(
            VirtioError::DevNotSupported(dev_id),
        ));
    }

    // Verify the device-ID to find the network card
    match registers.get_device_id() {
        #[cfg(feature = "tcp")]
        DevId::VIRTIO_DEV_ID_NET => {
            match VirtioNetDriver::init(dev_id, registers, irq_no) {
                Ok(virt_net_drv) => {
                    info!("Virtio network driver initialized.");
                    // Install interrupt handler
                    irq_install_handler(irq_no, network_irqhandler, "virtio_net");
                    Ok(VirtioDriver::Network(virt_net_drv))
                }
                Err(virtio_error) => {
                    error!("Virtio network driver could not be initialized with device");
                    Err(DriverError::InitVirtioDevFail(virtio_error))
                }
            }
        }
        #[cfg(feature = "fs")]
        DevId::VIRTIO_DEV_ID_BLK => {
            match VirtioBlkDriver::init(dev_id, registers, irq_no) {
                Ok(virt_blk_drv) => {
                    info!("Virtio blk driver initialized.");
                    // Install interrupt handler
                    irq_install_handler(irq_no, blk_irqhandler, "virtio_blk");
                    Ok(VirtioDriver::Blk(virt_blk_drv))
                }
                Err(virtio_error) => {
                    error!("Virtio blk driver could not be initialized with device");
                    Err(DriverError::InitVirtioDevFail(virtio_error))
                }
            }
        }
        _ => {
            error!(
                "Device with id {:?} is currently not supported!",
                registers.get_device_id()
            );
            // Return Driver error inidacting device is not supported
            Err(DriverError::InitVirtioDevFail(
                VirtioError::DevNotSupported(dev_id),
            ))
        }
    }
}

register_structs! {
  /// The Layout of MMIO Device
  #[repr(C, align(4))]
  #[allow(non_snake_case)]
  pub MmioRegisterLayout {
    (0x000 => pub magic_value: ReadOnly<u32>),
    (0x004 => pub version: ReadOnly<u32>),
    (0x008 => pub device_id: ReadOnly<u32>),
    (0x00c => pub vendor_id: ReadOnly<u32>),

    (0x010 => pub device_features: ReadOnly<u32>),
    (0x014 => pub device_features_sel: WriteOnly<u32>),
    (0x018 => _reserved0),
    (0x020 => pub driver_features: WriteOnly<u32>),
    (0x024 => pub driver_features_sel: WriteOnly<u32>),

    (0x028 => pub guest_page_size: WriteOnly<u32>), // legacy only
    (0x02c => _reserved_1),

    (0x030 => pub queue_sel: WriteOnly<u32>),
    (0x034 => pub queue_num_max: ReadOnly<u32>),
    (0x038 => pub queue_num: WriteOnly<u32>),
    (0x03c => pub queue_align: WriteOnly<u32>), // legacy only
    (0x040 => pub queue_pfn: ReadWrite<u32>), // legacy only
    (0x044 => pub queue_ready: ReadWrite<u32>),
    (0x048 => _reserved2),
    (0x050 => pub queue_notify: WriteOnly<u32>),
    (0x054 => _reserved3),

    (0x060 => pub interrupt_status: ReadOnly<u32>),
    (0x064 => pub interrupt_ack: WriteOnly<u32>),
    (0x068 => _reserved4),

    (0x070 => pub status: ReadWrite<u32>),
    (0x074 =>  _reserved5),

    (0x080 => pub queue_desc_low: WriteOnly<u32>),
    (0x084 => pub queue_desc_high: WriteOnly<u32>),
    (0x088 =>  _reserved6),
    (0x090 => pub queue_driver_low: WriteOnly<u32>),
    (0x094 => pub queue_driver_high: WriteOnly<u32>),
    (0x098 =>  _reserved7),
    (0x0a0 => pub queue_device_low: WriteOnly<u32>),
    (0x0a4 => pub queue_device_high: WriteOnly<u32>),
    (0x0a8 =>  _reserved8),

    (0x0fc => pub config_generation: ReadOnly<u32>),
    (0x100 => pub config: [ReadWrite<u32>; 3]),
    (0x200 => @END),
  }
}

impl MmioRegisterLayout {
    pub fn get_magic_value(&self) -> u32 {
        self.magic_value.get()
    }

    pub fn get_version(&self) -> u32 {
        self.version.get()
    }

    pub fn get_device_id(&self) -> DevId {
        DevId::from(self.device_id.get())
    }

    pub fn get_vendor_id(&self) -> u32 {
        self.vendor_id.get()
    }

    pub fn enable_queue(&self, sel: u32) {
        self.queue_sel.set(sel);
        self.queue_ready.set(1u32);
    }

    pub fn get_max_queue_size(&self, sel: u32) -> u32 {
        self.queue_sel.set(sel);
        self.queue_num_max.get()
    }

    pub fn set_queue_size(&self, sel: u32, size: u32) -> u32 {
        self.queue_sel.set(sel);
        let num_max = self.queue_num_max.get();
        if num_max >= size {
            self.queue_num.set(size);
            size
        } else {
            self.queue_num.set(num_max);
            num_max
        }
    }

    pub fn set_ring_addr(&self, sel: u32, addr: usize) {
        self.queue_sel.set(sel);
        self.queue_desc_low.set(addr as u32);
        self.queue_desc_high.set((addr >> 32) as u32);
    }

    pub fn set_drv_ctrl_addr(&self, sel: u32, addr: usize) {
        self.queue_sel.set(sel);
        self.queue_driver_low.set(addr as u32);
        self.queue_driver_high.set((addr >> 32) as u32);
    }

    pub fn set_dev_ctrl_addr(&self, sel: u32, addr: usize) {
        self.queue_sel.set(sel);
        self.queue_device_low.set(addr as u32);
        self.queue_device_high.set((addr >> 32) as u32);
    }

    pub fn get_queue_ready(&self, sel: u32) -> bool {
        self.queue_sel.set(sel);
        self.queue_ready.get() != 0
    }

    pub fn dev_features(&self) -> u64 {
        // Indicate device to show high 32 bits in device_feature field.
        // See Virtio specification v1.1. - 4.1.4.3
        self.device_features_sel.set(1u32);
        let mut dev_feat = u64::from(self.device_features.get()) << 32;

        // Indicate device to show low 32 bits in device_feature field.
        // See Virtio specification v1.1. - 4.1.4.3

        self.device_features_sel.set(0u32);

        // read low 32 bits of device features
        dev_feat |= u64::from(self.device_features.get());

        dev_feat
    }

    /// Write selected features into driver_select field.
    pub fn set_drv_features(&self, feats: u64) {
        let high: u32 = (feats >> 32) as u32;
        let low: u32 = feats as u32;

        // Indicate to device that driver_features field shows low 32 bits.
        // See Virtio specification v1.1. - 4.1.4.3
        self.driver_features_sel.set(0u32);

        // write low 32 bits of device features
        self.driver_features.set(low);

        // Indicate to device that driver_features field shows high 32 bits.
        // See Virtio specification v1.1. - 4.1.4.3
        self.driver_features_sel.set(1u32);

        // write high 32 bits of device features
        self.driver_features.set(high);
    }

    pub fn get_config(&self) -> [u32; 3] {
        // see Virtio specification v1.1 -  2.4.1
        loop {
            let before = self.config_generation.get();
            fence(Ordering::SeqCst);
            let config = [
                self.config[0].get(),
                self.config[1].get(),
                self.config[2].get(),
            ];
            fence(Ordering::SeqCst);
            let after = self.config_generation.get();
            fence(Ordering::SeqCst);

            if before == after {
                return config;
            }
        }
    }

    pub fn print_information(&self) {
        infoheader!(" MMIO RREGISTER LAYOUT INFORMATION ");

        infoentry!("Device version", "{:#X}", self.get_version());
        infoentry!("Device ID", "{:?}", self.get_device_id());
        infoentry!("Vendor ID", "{:#X}", self.get_vendor_id());
        infoentry!("Device Features", "{:#X}", self.dev_features());
        infoentry!("Interrupt status", "{:#X}", self.interrupt_status.get());
        infoentry!("Device status", "{:#X}", self.status.get());
        infoentry!("Configuration space", "{:#X?}", self.get_config());

        infofooter!();
    }
}
