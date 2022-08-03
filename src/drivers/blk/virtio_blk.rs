use alloc::rc::Rc;
use core::slice;

use crate::drivers::blk::{constants, BlkInterface};
use crate::drivers::blk::mmio::BlkDevCfgRaw;
use crate::drivers::blk::constants::{RespStatus, BLK_SIZE, ReqType};
use crate::drivers::blk::virtio_blk::error::VirtioBlkError;

use crate::drivers::virtio::transport::mmio::{IsrStatus, NotifCfg, ComCfg};
use crate::drivers::virtio::virtqueue::{
    AsSliceU8, BufferToken, Virtq, VqIndex, VqSize, VqType,
};
use crate::drivers::virtio::VIRTIO_MAX_QUEUE_SIZE;
use crate::drivers::virtio::features::Features;

/// Virtio specification v1.1. - 5.2.6
/// The driver queues requests to the virtqueue, and they are used by the device (not necessarily in order).
/// struct virtio_blk_req {
///     le32 type;
///     le32 reserved;
///     le64 sector;
///     u8 data[];
///     u8 status;
/// };

/// Request of a VirtioIOBlk request.
#[derive(Debug)]
#[repr(C)]
pub struct VirtioBlkReqHdr {
    req_type: ReqType,
    reserved: u32,
    sector: u64,
}
/// Buffer is provided by the caller.
/// Response of a VirtIOBlk request.
#[derive(Debug)]
#[repr(C)]
pub struct VirtioBlkBlkResp {
    status: RespStatus,
}

impl Default for VirtioBlkBlkResp {
    fn default() -> Self {
        VirtioBlkBlkResp {
            status: RespStatus::_NotReady,
        }
    }
}

impl AsSliceU8 for VirtioBlkReqHdr {}
impl AsSliceU8 for VirtioBlkBlkResp {}

pub struct ReqQueue {
    vq: Option<Rc<Virtq>>,
}

impl ReqQueue {
    pub fn new(vq: Option<Rc<Virtq>>) -> Self {
        Self { vq }
    }
    #[allow(dead_code)]
    fn enable_notifs(&self) {
        self.vq.as_ref().unwrap().enable_notifs();
    }

    #[allow(dead_code)]
    fn disable_notifs(&self) {
        self.vq.as_ref().unwrap().disable_notifs();
    }

    fn poll(&self) {
        self.vq.as_ref().unwrap().poll();
    }

    fn setup_vp(&mut self, vq: Virtq, _dev_cfg: &BlkDevCfg) {
        // Safe virtqueue
        if self.vq.is_none() {
            self.vq = Some(Rc::new(vq));
        } else {
            panic!("Virtio Blk request has been initialized!!!");
        }
    }

    /// Returns a buffertoken of required buff size for block write.
    ///
    /// OR returns None, if no Buffertoken could be generated
    fn get_block_read_tkn(
        &mut self,
        sector: usize,
        buf: &mut [u8],
        resp: &mut [u8],
    ) -> Option<BufferToken> {
        // Virtio specification v1.1. - 5.2.6.1 point 5.
        //      The length of data MUST be a multiple of 512 bytes for VIRTIO_BLK_T_IN and VIRTIO_BLK_T_OUT requests
        assert_eq!(buf.len() % BLK_SIZE, 0);

        let req = VirtioBlkReqHdr {
            req_type: ReqType::VIRTIO_BLK_T_IN,
            reserved: 0,
            sector: sector as u64,
        };

        match self.vq.as_ref().unwrap().prep_buffer_from_existing_memory(
            Rc::clone(&self.vq.as_ref().unwrap()),
            &[req.as_slice_u8()],
            &[buf, resp],
        ) {
            Ok(tkn) => Some(tkn),
            Err(_) => {
                // Here it is possible if multiple queues are enabled to get another buffertoken from them!
                // Info the queues are disabled upon initialization and should be enabled somehow!
                None
            }
        }
    }

    /// Returns a buffertoken of required buff size for block read.
    ///
    /// OR returns None, if no Buffertoken could be generated
    fn get_block_write_tkn(
        &mut self,
        sector: usize,
        buf: &mut [u8],
        resp: &mut [u8],
    ) -> Option<BufferToken> {
        // Virtio specification v1.1. - 5.2.6.1 point 5.
        //      The length of data MUST be a multiple of 512 bytes for VIRTIO_BLK_T_IN and VIRTIO_BLK_T_OUT requests
        assert_eq!(buf.len() % BLK_SIZE, 0);

        let req = VirtioBlkReqHdr {
            req_type: ReqType::VIRTIO_BLK_T_OUT,
            reserved: 0,
            sector: sector as u64,
        };

        match self.vq.as_ref().unwrap().prep_buffer_from_existing_memory(
            Rc::clone(&self.vq.as_ref().unwrap()),
            &[req.as_slice_u8(), buf],
            &[resp],
        ) {
            Ok(tkn) => Some(tkn),
            Err(_) => {
                // Here it is possible if multiple queues are enabled to get another buffertoken from them!
                // Info the queues are disabled upon initialization and should be enabled somehow!
                None
            }
        }
    }
}

/// A wrapper struct for the raw configuration structure.
/// Handling the right access to fields, as some are read-only
/// for the driver.
pub struct BlkDevCfg {
    pub raw: &'static BlkDevCfgRaw,
    pub dev_id: u16,

    // Feature booleans
    pub features: u64,
}

pub struct VirtioBlkDriver {
    pub(super) dev_cfg: BlkDevCfg,
    pub(super) com_cfg: ComCfg,
    pub(super) isr_stat: IsrStatus,
    pub(super) notif_cfg: NotifCfg,

    pub(super) request_vq: ReqQueue,
    pub(super) irq: u8,
}

impl BlkInterface for VirtioBlkDriver {
    fn read_block(&mut self, sector: usize, count: usize, buf: usize) {
        debug!(
            "read_block() sector {} count {} buf 0x{:x}",
            sector, count, buf
        );
        let len = count * BLK_SIZE;
        let mut buf = unsafe { slice::from_raw_parts_mut(buf as *mut u8, len) };
        let mut resp = VirtioBlkBlkResp::default();
        match self
            .request_vq
            .get_block_read_tkn(sector, &mut buf, &mut resp.as_slice_u8_mut())
        {
            Some(buff_tkn) => {
                // Send read request.
                match buff_tkn.provide().dispatch_blocking() {
                    Ok(_) => {}
                    Err(err) => {
                        warn!(
                            "read_block() sector {} count {} buff_tkn dispatch error {:?}",
                            sector, count, err
                        );
                        return;
                    }
                }
            }
            None => {
                warn!(
                    "read_block() sector {} count {} get empty block_read_tkn",
                    sector, count
                );
                return;
            }
        }
        self.request_vq.poll();
        match resp.status {
            RespStatus::Ok => {
                debug!("read_block() resp status ok");
            }
            _ => {
                warn!("read_block() resp status {:?}", resp.status);
            }
        }
    }

    fn write_block(&mut self, sector: usize, count: usize, buf: usize) {
        debug!(
            "write_block() sector {} count {} buf 0x{:x}",
            sector, count, buf
        );
        let len = count * BLK_SIZE;
        let mut buf = unsafe { slice::from_raw_parts_mut(buf as *mut u8, len) };
        let mut resp = VirtioBlkBlkResp::default();
        match self
            .request_vq
            .get_block_write_tkn(sector, &mut buf, &mut resp.as_slice_u8_mut())
        {
            Some(buff_tkn) => {
                // Send write request.
                match buff_tkn.provide().dispatch_blocking() {
                    Ok(_) => {}
                    Err(err) => {
                        warn!(
                            "write_block() sector {} count {} buff_tkn dispatch error {:?}",
                            sector, count, err
                        );
                        return;
                    }
                }
            }
            None => {
                warn!(
                    "write_block() sector {} count {} get empty block_read_tkn",
                    sector, count
                );
                return;
            }
        }
        self.request_vq.poll();
        match resp.status {
            RespStatus::Ok => {
                debug!("write_block() resp status ok");
            }
            _ => {
                warn!("write_block() resp status {:?}", resp.status);
            }
        }
    }
    fn handle_interrupt(&mut self) -> bool {
        trace!("handle interrupt 32 + {}", self.irq);

        let result = if self.isr_stat.is_interrupt() {
            // handle incoming interrupts
            true
        } else if self.isr_stat.is_cfg_change() {
            info!("Configuration changes are not possible! Aborting");
            todo!("Implement possibiity to change config on the fly...")
        } else {
            false
        };

        self.isr_stat.acknowledge();

        result
    }
}

impl VirtioBlkDriver {
    pub fn get_dev_id(&self) -> u16 {
        self.dev_cfg.dev_id
    }

    pub fn set_failed(&mut self) {
        self.com_cfg.set_failed();
    }
    /// Initiallizes the device in adherence to specification. Returns Some(VirtioNetError)
    /// upon failure and None in case everything worked as expected.
    ///
    /// See Virtio specification v1.1. - 3.1.1.
    ///                      and v1.1. - 5.1.5
    pub fn init_dev(&mut self) -> Result<(), VirtioBlkError> {
        // Reset
        self.com_cfg.reset_dev();

        // Sets the ACKNOWLEDGE bit in the device status field.
        // Indiacte device, that OS noticed it
        self.com_cfg.ack_dev();

        // Sets the DRIVER bit in the device status field.
        // Indicate device, that driver is able to handle it
        self.com_cfg.set_drv();

        //Todo: add negotiate_features like network do.
        let feature = u64::from(Features::VIRTIO_F_VERSION_1)
        | u64::from(constants::Features::VIRTIO_BLK_F_SEG_MAX)
        | u64::from(constants::Features::VIRTIO_BLK_F_GEOMETRY)
        // If the VIRTIO_BLK_F_BLK_SIZE feature is negotiated, blk_size can be read to determine the optimal
        // sector size for the driver to use. This does not affect the units used in the protocol (always 512 bytes),
        // but awareness of the correct value can affect performance.
        | u64::from(constants::Features::VIRTIO_BLK_F_BLK_SIZE)
        // If the VIRTIO_BLK_F_TOPOLOGY feature is negotiated, the fields in the topology struct can be read
        // to determine the physical block size and optimal I/O lengths for the driver to use. This also does not 
        // affect the units in the protocol, only performance.
        | u64::from(constants::Features::VIRTIO_BLK_F_TOPOLOGY);

        self.com_cfg.set_drv_features(feature);

        // Indicates the device, that the current feature set is final for the driver
        // and will not be changed.
        self.com_cfg.features_ok();

        match self.dev_spec_init() {
            Ok(_) => info!(
                "Device specific initialization for Virtio network device {:x} finished",
                self.dev_cfg.dev_id
            ),
            Err(vnet_err) => return Err(vnet_err),
        }
        // Sets the DRIVER_OK bit in the device status field.
        // At this point the device is "live"
        self.com_cfg.drv_ok();

        Ok(())
    }

    /// Device Specific initialization according to Virtio specifictation v1.1. - 5.1.5
    fn dev_spec_init(&mut self) -> Result<(), VirtioBlkError> {
        match self.virtqueue_init() {
            Ok(_) => info!("Block driver successfully initialized virtqueues."),
            Err(vnet_err) => return Err(vnet_err),
        }
        Ok(())
    }

    /// Initialize virtqueues via the queue interface and populates receiving queues
    fn virtqueue_init(&mut self) -> Result<(), VirtioBlkError> {
        let index = 0 as u32;
        let vq = Virtq::new(
            &mut self.com_cfg,
            &self.notif_cfg,
            VqSize::from(VIRTIO_MAX_QUEUE_SIZE),
            VqType::Split,
            VqIndex::from(index),
            self.dev_cfg.features.into(),
        );
        // Interrupt for comunicating that a block request left, is not needed.
        vq.disable_notifs();

        self.request_vq.setup_vp(vq, &self.dev_cfg);
        Ok(())
    }
}

/// Error module of virtios block driver. Containing the (VirtioBlkError)[VirtioBlkError]
/// enum.
pub mod error {
    /// Network drivers error enum.
    #[derive(Debug, Copy, Clone)]
    pub enum VirtioBlkError {
        General,
        NoDevCfg(u16),
        NoComCfg(u16),
        NoIsrCfg(u16),
        NoNotifCfg(u16),
        /// Indicates that an operation for finished Transfers, was performed on
        /// an ongoing transfer
        ProcessOngoing,
        Unknown,
    }
}
