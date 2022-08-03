use crate::drivers::blk::virtio_blk::{BlkDevCfg, VirtioBlkDriver};
use crate::drivers::virtio::transport::mmio::{MmioRegisterLayout, IsrStatus, NotifCfg, ComCfg};
use crate::drivers::virtio::error::VirtioError;

use crate::drivers::blk::virtio_blk::ReqQueue;

/// Virtio's blk device configuration structure.
/// See specification v1.1. - 5.1.4
///
#[repr(C)]
struct VirtioBlkGeometry {
    cylinders: u16,
    heads: u8,
    sectors: u8,
}
#[repr(C)]
struct VirtioBlkTopology {
    // # of logical blocks per physical block (log2)
    physical_block_exp: u8,
    // offset of first aligned logical block
    alignment_offset: u8,
    // suggested minimum I/O size in blocks
    min_io_size: u16,
    // optimal (suggested maximum) I/O size in blocks
    opt_io_size: u32,
}
#[repr(C)]
pub struct BlkDevCfgRaw {
    capacity: u32,
    size_max: u32,
    seg_max: u32,
    geometry: VirtioBlkGeometry,
    blk_size: u32,
    topology: VirtioBlkTopology,
    writeback: u8,
    unused0: [u8; 3],
    max_discard_sectors: u32,
    max_discard_seg: u32,
    discard_sector_alignment: u32,
    max_write_zeroes_sectors: u32,
    max_write_zeroes_seg: u32,
    write_zeroes_may_unmap: u8,
    unused1: [u8; 3],
}

impl VirtioBlkDriver {
    pub fn new(
        dev_id: u16,
        registers: &'static mut MmioRegisterLayout,
        irq: u8,
    ) -> Result<Self, VirtioError> {
        let dev_cfg_raw: &'static BlkDevCfgRaw =
            unsafe { &*(((registers as *const _ as usize) + 0xFC) as *const BlkDevCfgRaw) };
        let dev_cfg = BlkDevCfg {
            raw: dev_cfg_raw,
            dev_id,
            features: 0,
        };
        let isr_stat = IsrStatus::new(registers);
        let notif_cfg = NotifCfg::new(registers);

        Ok(VirtioBlkDriver {
            dev_cfg,
            com_cfg: ComCfg::new(registers, 1),
            isr_stat,
            notif_cfg,
            irq,
            request_vq: ReqQueue::new(None),
        })
    }
    pub fn init(
        dev_id: u16,
        registers: &'static mut MmioRegisterLayout,
        irq_no: u32,
    ) -> Result<VirtioBlkDriver, VirtioError> {
        if let Ok(mut drv) = VirtioBlkDriver::new(dev_id, registers, irq_no.try_into().unwrap()) {
            match drv.init_dev() {
                Err(error_code) => Err(VirtioError::BlkDriver(error_code)),
                _ => {
                    // drv.print_information();
                    Ok(drv)
                }
            }
        } else {
            error!("Unable to create Driver. Aborting!");
            Err(VirtioError::Unknown)
        }
    }
}
