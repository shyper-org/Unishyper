//! A module containing a virtio network driver.
//!
//! The module contains ...
// use crate::arch::kernel::percore::increment_irq_counter;
use crate::drivers::net::NetworkInterface;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::mem;
use core::result::Result;
use core::{cell::RefCell, cmp::Ordering};

use crate::drivers::net::virtio_mmio::NetDevCfgRaw;
use crate::drivers::virtio::transport::mmio::{ComCfg, IsrStatus, NotifCfg};
use crate::drivers::virtio::virtqueue::{
    AsSliceU8, BuffSpec, BufferToken, Bytes, Transfer, Virtq, VqIndex, VqSize, VqType,
};
use crate::drivers::virtio::VIRTIO_MAX_QUEUE_SIZE;

use crate::drivers::net::constants::{FeatureSet, Features, NetHdrGSO, Status, MAX_NUM_VQ};
use self::error::VirtioNetError;

use super::netwakeup;

pub const ETH_HDR: usize = 14usize;

/// A wrapper struct for the raw configuration structure.
/// Handling the right access to fields, as some are read-only
/// for the driver.
pub struct NetDevCfg {
    pub raw: &'static NetDevCfgRaw,
    pub dev_id: u16,

    // Feature booleans
    pub features: FeatureSet,
}

#[derive(Debug)]
#[repr(C)]
pub struct VirtioNetHdr {
    flags: u8,
    gso_type: u8,
    /// Ethernet + IP + tcp/udp hdrs
    hdr_len: u16,
    /// Bytes to append to hdr_len per frame
    gso_size: u16,
    /// Position to start checksumming from
    csum_start: u16,
    /// Offset after that to place checksum
    csum_offset: u16,
    /// Number of buffers this Packet consists of
    num_buffers: u16,
}

// Using the default implementation of the trait for VirtioNetHdr
impl AsSliceU8 for VirtioNetHdr {}

impl VirtioNetHdr {
    pub fn get_tx_hdr() -> VirtioNetHdr {
        VirtioNetHdr {
            flags: 0,
            gso_type: NetHdrGSO::VIRTIO_NET_HDR_GSO_NONE.into(),
            hdr_len: 0,
            gso_size: 0,
            csum_start: 0,
            csum_offset: 0,
            num_buffers: 0,
        }
    }

    pub fn get_rx_hdr() -> VirtioNetHdr {
        VirtioNetHdr {
            flags: 0,
            gso_type: 0,
            hdr_len: 0,
            gso_size: 0,
            csum_start: 0,
            csum_offset: 0,
            num_buffers: 0,
        }
    }
}

pub struct CtrlQueue(Option<Rc<Virtq>>);

impl CtrlQueue {
    pub fn new(vq: Option<Rc<Virtq>>) -> Self {
        CtrlQueue(vq)
    }
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum CtrlClass {
    VIRTIO_NET_CTRL_RX = 1 << 0,
    VIRTIO_NET_CTRL_MAC = 1 << 1,
    VIRTIO_NET_CTRL_VLAN = 1 << 2,
    VIRTIO_NET_CTRL_ANNOUNCE = 1 << 3,
    VIRTIO_NET_CTRL_MQ = 1 << 4,
}

impl From<CtrlClass> for u8 {
    fn from(val: CtrlClass) -> Self {
        match val {
            CtrlClass::VIRTIO_NET_CTRL_RX => 1 << 0,
            CtrlClass::VIRTIO_NET_CTRL_MAC => 1 << 1,
            CtrlClass::VIRTIO_NET_CTRL_VLAN => 1 << 2,
            CtrlClass::VIRTIO_NET_CTRL_ANNOUNCE => 1 << 3,
            CtrlClass::VIRTIO_NET_CTRL_MQ => 1 << 4,
        }
    }
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum RxCmd {
    VIRTIO_NET_CTRL_RX_PROMISC = 1 << 0,
    VIRTIO_NET_CTRL_RX_ALLMULTI = 1 << 1,
    VIRTIO_NET_CTRL_RX_ALLUNI = 1 << 2,
    VIRTIO_NET_CTRL_RX_NOMULTI = 1 << 3,
    VIRTIO_NET_CTRL_RX_NOUNI = 1 << 4,
    VIRTIO_NET_CTRL_RX_NOBCAST = 1 << 5,
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum MacCmd {
    VIRTIO_NET_CTRL_MAC_TABLE_SET = 1 << 0,
    VIRTIO_NET_CTRL_MAC_ADDR_SET = 1 << 1,
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum VlanCmd {
    VIRTIO_NET_CTRL_VLAN_ADD = 1 << 0,
    VIRTIO_NET_CTRL_VLAN_DEL = 1 << 1,
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum AnceCmd {
    VIRTIO_NET_CTRL_ANNOUNCE_ACK = 1 << 0,
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum MqCmd {
    VIRTIO_NET_CTRL_MQ_VQ_PAIRS_SET = 1 << 0,
    VIRTIO_NET_CTRL_MQ_VQ_PAIRS_MIN = 1 << 1,
    VIRTIO_NET_CTRL_MQ_VQ_PAIRS_MAX = 0x80,
}

pub struct RxQueues {
    vqs: Vec<Rc<Virtq>>,
    poll_queue: Rc<RefCell<VecDeque<Transfer>>>,
    is_multi: bool,
}

impl RxQueues {
    pub fn new(
        vqs: Vec<Rc<Virtq>>,
        poll_queue: Rc<RefCell<VecDeque<Transfer>>>,
        is_multi: bool,
    ) -> Self {
        Self {
            vqs,
            poll_queue,
            is_multi,
        }
    }

    /// Takes care if handling packets correctly which need some processing after being received.
    /// This currently include nothing. But in the future it might include among others::
    /// * Calculating missing checksums
    /// * Merging receive buffers, by simply checking the poll_queue (if VIRTIO_NET_F_MRG_BUF)
    fn post_processing(transfer: Transfer) -> Result<Transfer, VirtioNetError> {
        if transfer.poll() {
            // Here we could implement all features.
            Ok(transfer)
        } else {
            warn!("Unfinished transfer in post processing. Returning buffer to queue. This will need explicit cleanup.");
            transfer.close();
            Err(VirtioNetError::ProcessOngoing)
        }
    }

    /// Adds a given queue to the underlying vector and populates the queue with RecvBuffers.
    ///
    /// Queues are all populated according to Virtio specification v1.1. - 5.1.6.3.1
    fn add(&mut self, vq: Virtq, dev_cfg: &NetDevCfg) {
        // Safe virtqueue
        let rc_vq = Rc::new(vq);
        let vq = &rc_vq;

        // VIRTIO_NET_F_GUEST_TSO4 VIRTIO_NET_F_GUEST_TSO6 VIRTIO_NET_F_GUEST_UFO
        // If above features not set, buffers must be at least 1526 bytes large.
        // See Virtio specification v1.1 - 5.1.6.3.1

        let buff_def = [
            Bytes::new(mem::size_of::<VirtioNetHdr>()).unwrap(),
            Bytes::new(1514).unwrap(),
        ];
        let spec = if dev_cfg
            .features
            .is_feature(Features::VIRTIO_F_RING_INDIRECT_DESC)
        {
            BuffSpec::Indirect(&buff_def)
        } else {
            BuffSpec::Single(Bytes::new(mem::size_of::<VirtioNetHdr>() + 1514).unwrap())
        };

        let num_buff: u16 = vq.size().into();

        debug!("RX Queues add, num buff {}", num_buff);
        for _ in 0..num_buff {
            let buff_tkn = match vq.prep_buffer(Rc::clone(vq), None, Some(spec.clone())) {
                Ok(tkn) => tkn,
                Err(_vq_err) => {
                    error!("Setup of network queue failed, which should not happen!");
                    panic!("setup of network queue failed!");
                }
            };

            // BufferTokens are directly provided to the queue
            // TransferTokens are directly dispatched
            // Transfers will be awaited at the queue
            buff_tkn
                .provide()
                .dispatch_await(Rc::clone(&self.poll_queue), false);
        }

        // Safe virtqueue
        self.vqs.push(rc_vq);

        if self.vqs.len() > 1 {
            self.is_multi = true;
        }
    }

    fn get_next(&mut self) -> Option<Transfer> {
        let transfer = self.poll_queue.borrow_mut().pop_front();

        transfer.or_else(|| {
            // Check if any not yet provided transfers are in the queue.
            self.poll();

            self.poll_queue.borrow_mut().pop_front()
        })
    }

    fn poll(&self) {
        if self.is_multi {
            for vq in &self.vqs {
                vq.poll();
            }
        } else {
            self.vqs[0].poll();
        }
    }

    fn enable_notifs(&self) {
        if self.is_multi {
            for vq in &self.vqs {
                vq.enable_notifs();
            }
        } else {
            self.vqs[0].enable_notifs();
        }
    }

    fn disable_notifs(&self) {
        if self.is_multi {
            for vq in &self.vqs {
                vq.disable_notifs();
            }
        } else {
            self.vqs[0].disable_notifs();
        }
    }
}

/// Structure which handles transmission of packets and delegation
/// to the respective queue structures.
pub struct TxQueues {
    vqs: Vec<Rc<Virtq>>,
    poll_queue: Rc<RefCell<VecDeque<Transfer>>>,
    ready_queue: Vec<BufferToken>,
    /// Indicates, whether the Driver/Device are using multiple
    /// queues for communication.
    is_multi: bool,
}

impl TxQueues {
    pub fn new(
        vqs: Vec<Rc<Virtq>>,
        poll_queue: Rc<RefCell<VecDeque<Transfer>>>,
        ready_queue: Vec<BufferToken>,
        is_multi: bool,
    ) -> Self {
        Self {
            vqs,
            poll_queue,
            ready_queue,
            is_multi,
        }
    }
    #[allow(dead_code)]
    fn enable_notifs(&self) {
        if self.is_multi {
            for vq in &self.vqs {
                vq.enable_notifs();
            }
        } else {
            self.vqs[0].enable_notifs();
        }
    }

    #[allow(dead_code)]
    fn disable_notifs(&self) {
        if self.is_multi {
            for vq in &self.vqs {
                vq.disable_notifs();
            }
        } else {
            self.vqs[0].disable_notifs();
        }
    }

    fn poll(&self) {
        if self.is_multi {
            for vq in &self.vqs {
                vq.poll();
            }
        } else {
            self.vqs[0].poll();
        }
    }

    fn add(&mut self, vq: Virtq, dev_cfg: &NetDevCfg) {
        // Safe virtqueue
        self.vqs.push(Rc::new(vq));
        if self.vqs.len() == 1 {
            // Unwrapping is safe, as one virtq will be definitely in the vector.
            let vq = self.vqs.get(0).unwrap();

            // Virtio specification v1.1. - 5.1.6.2 point 5.
            //      Header and data are added as ONE output descriptor to the transmitvq.
            //      Hence we are interpreting this, as the fact, that send packets must be inside a single descriptor.
            // As usize is currently safe as the minimal usize is defined as 16bit in rust.
            let buff_def = Bytes::new(
                mem::size_of::<VirtioNetHdr>() + (dev_cfg.raw.get_mtu() as usize) + ETH_HDR,
            )
            .unwrap();
            let spec = BuffSpec::Single(buff_def);

            let num_buff: u16 = vq.size().into();

            debug!("TxQueues add, num buff {}", num_buff);
            for _ in 0..num_buff {
                self.ready_queue.push(
                    vq.prep_buffer(Rc::clone(vq), Some(spec.clone()), None)
                        .unwrap()
                        .write_seq(Some(VirtioNetHdr::get_tx_hdr()), None::<VirtioNetHdr>)
                        .unwrap(),
                )
            }
        } else {
            self.is_multi = true;
            // Currently we are doing nothing with the additional queues. They are inactive and might be used in the
            // future
        }
    }

    /// Returns either a buffertoken and the corresponding index of the
    /// virtqueue it is coming from. (Index in the TxQueues.vqs vector)
    ///
    /// OR returns None, if no Buffertoken could be generated
    fn get_tkn(&mut self, len: usize) -> Option<(BufferToken, usize)> {
        // Check all ready token, for correct size.
        // Drop token if not so
        //
        // All Tokens inside the ready_queue are coming from the main queue with index 0.
        while let Some(mut tkn) = self.ready_queue.pop() {
            let (send_len, _) = tkn.len();

            match send_len.cmp(&len) {
                Ordering::Less => {}
                Ordering::Equal => return Some((tkn, 0)),
                Ordering::Greater => {
                    tkn.restr_size(Some(len), None).unwrap();
                    return Some((tkn, 0));
                }
            }
        }

        if self.poll_queue.borrow().is_empty() {
            self.poll();
        }

        while let Some(transfer) = self.poll_queue.borrow_mut().pop_back() {
            let mut tkn = transfer.reuse().unwrap();
            let (send_len, _) = tkn.len();

            match send_len.cmp(&len) {
                Ordering::Less => {}
                Ordering::Equal => return Some((tkn, 0)),
                Ordering::Greater => {
                    tkn.restr_size(Some(len), None).unwrap();
                    return Some((tkn, 0));
                }
            }
        }

        // As usize is currently safe as the minimal usize is defined as 16bit in rust.
        let spec = BuffSpec::Single(Bytes::new(len).unwrap());

        match self.vqs[0].prep_buffer(Rc::clone(&self.vqs[0]), Some(spec), None) {
            Ok(tkn) => Some((tkn, 0)),
            Err(_) => {
                // Here it is possible if multiple queues are enabled to get another buffertoken from them!
                // Info the queues are disabled upon initialization and should be enabled somehow!
                None
            }
        }
    }
}

/// Virtio network driver struct.
///
/// Struct allows to control devices virtqueues as also
/// the device itself.
pub struct VirtioNetDriver {
    pub(super) dev_cfg: NetDevCfg,
    pub(super) com_cfg: ComCfg,
    pub(super) isr_stat: IsrStatus,
    pub(super) notif_cfg: NotifCfg,

    pub(super) ctrl_vq: CtrlQueue,
    pub(super) recv_vqs: RxQueues,
    pub(super) send_vqs: TxQueues,

    pub(super) num_vqs: u16,
    pub(super) irq: u8,
}

impl NetworkInterface for VirtioNetDriver {
    /// Returns the mac address of the device.
    /// If VIRTIO_NET_F_MAC is not set, the function panics currently!
    fn get_mac_address(&self) -> [u8; 6] {
        if self.dev_cfg.features.is_feature(Features::VIRTIO_NET_F_MAC) {
            self.dev_cfg.raw.get_mac()
        } else {
            unreachable!("Currently VIRTIO_NET_F_MAC must be negotiated!")
        }
    }

    /// Returns the current MTU of the device.
    /// Currently, if VIRTIO_NET_F_MAC is not set
    //  MTU is set static to 1500 bytes.
    fn get_mtu(&self) -> u16 {
        if self.dev_cfg.features.is_feature(Features::VIRTIO_NET_F_MTU) {
            self.dev_cfg.raw.get_mtu()
        } else {
            1500
        }
    }

    /// Provides the "user-space" with a pointer to usable memory.
    ///
    /// Therefore the driver checks if a free BufferToken is in its TxQueues struct.
    /// If one is found, the function does return a pointer to the memory area, where
    /// the "user-space" can write to and a raw pointer to the token in order to provide
    /// it to the queue after the "user-space" driver has written to the buffer.
    ///
    /// If not BufferToken is found the functions returns an error.
    fn get_tx_buffer(&mut self, len: usize) -> Result<(*mut u8, usize), ()> {
        // Adding virtio header size to length.
        let len = len + core::mem::size_of::<VirtioNetHdr>();

        match self.send_vqs.get_tkn(len) {
            Some((mut buff_tkn, _vq_index)) => {
                let (send_ptrs, _) = buff_tkn.raw_ptrs();
                // Currently we have single Buffers in the TxQueue of size: MTU + ETH_HDR + VIRTIO_NET_HDR
                // see TxQueue.add()
                let (buff_ptr, _) = send_ptrs.unwrap()[0];

                // Do not show user-space memory for VirtioNetHdr.
                let buff_ptr = unsafe {
                    buff_ptr.offset(isize::try_from(core::mem::size_of::<VirtioNetHdr>()).unwrap())
                };

                Ok((buff_ptr, Box::into_raw(Box::new(buff_tkn)) as usize))
            }
            None => Err(()),
        }
    }

    fn free_tx_buffer(&self, token: usize) {
        unsafe { drop(Box::from_raw(token as *mut BufferToken)) }
    }

    fn send_tx_buffer(&mut self, tkn_handle: usize, _len: usize) -> Result<(), ()> {
        // This does not result in a new assignment, or in a drop of the BufferToken, which
        // would be dangerous, as the memory is freed then.
        let tkn = *unsafe { Box::from_raw(tkn_handle as *mut BufferToken) };

        tkn.provide()
            .dispatch_await(Rc::clone(&self.send_vqs.poll_queue), false);

        Ok(())
    }

    fn has_packet(&self) -> bool {
        self.recv_vqs.poll();
        !self.recv_vqs.poll_queue.borrow().is_empty()
    }

    fn receive_rx_buffer(&mut self) -> Result<(&'static mut [u8], usize), ()> {
        match self.recv_vqs.get_next() {
            Some(transfer) => {
                let transfer = match RxQueues::post_processing(transfer) {
                    Ok(trf) => trf,
                    Err(vnet_err) => {
                        error!("Post processing failed. Err: {:?}", vnet_err);
                        return Err(());
                    }
                };

                let (_, recv_data_opt) = transfer.as_slices().unwrap();
                let mut recv_data = recv_data_opt.unwrap();

                // If the given length is zero, we currently fail.
                if recv_data.len() == 2 {
                    let recv_payload = recv_data.pop().unwrap();
                    // Create static reference for the user-space
                    // As long as we keep the Transfer in a raw reference this reference is static,
                    // so this is fine.
                    // let recv_ref = (recv_payload as *const [u8]) as *mut [u8];
                    // let ref_data: &'static mut [u8] = unsafe { &*(recv_ref) };
                    let recv_ref = (recv_payload as *const [u8]) as *mut u8;
                    let ref_data: &'static mut [u8] =
                        unsafe { core::slice::from_raw_parts_mut(recv_ref, recv_payload.len()) };
                    let raw_transfer = Box::into_raw(Box::new(transfer));
                    debug!(
                        "receive_rx_buffer() get transfer len == 2, raw_transfer {:x}",
                        raw_transfer as usize
                    );
                    Ok((ref_data, raw_transfer as usize))
                } else if recv_data.len() == 1 {
                    let packet = recv_data.pop().unwrap();
                    let payload_ptr =
                        (&packet[mem::size_of::<VirtioNetHdr>()] as *const u8) as *mut u8;

                    let ref_data: &'static mut [u8] = unsafe {
                        core::slice::from_raw_parts_mut(
                            payload_ptr,
                            packet.len() - mem::size_of::<VirtioNetHdr>(),
                        )
                    };
                    let raw_transfer = Box::into_raw(Box::new(transfer));
                    // debug!("receive_rx_buffer() get transfer len == 1, raw_transfer {:x}", raw_transfer as usize);

                    Ok((ref_data, raw_transfer as usize))
                } else {
                    error!("Empty transfer, or with wrong buffer layout. Reusing and returning error to user-space network driver...");
                    transfer
                        .reuse()
                        .unwrap()
                        .write_seq(None::<VirtioNetHdr>, Some(VirtioNetHdr::get_rx_hdr()))
                        .unwrap()
                        .provide()
                        .dispatch_await(Rc::clone(&self.recv_vqs.poll_queue), false);

                    Err(())
                }
            }
            None => Err(()),
        }
    }

    // Tells driver, that buffer is consumed and can be deallocated
    fn rx_buffer_consumed(&mut self, trf_handle: usize) {
        unsafe {
            let transfer = *Box::from_raw(trf_handle as *mut Transfer);

            // Reuse transfer directly
            transfer
                .reuse()
                .unwrap()
                .provide()
                .dispatch_await(Rc::clone(&self.recv_vqs.poll_queue), false);
        }
    }

    fn set_polling_mode(&mut self, value: bool) {
        if value {
            self.disable_interrupts()
        } else {
            self.enable_interrupts()
        }
    }

    fn handle_interrupt(&mut self) -> bool {
        trace!("handle interrupt 32 + {}", self.irq);

        let result = if self.isr_stat.is_interrupt() {
            // handle incoming packets
            netwakeup();
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

// Backend-independent interface for Virtio network driver
impl VirtioNetDriver {
    pub fn get_dev_id(&self) -> u16 {
        self.dev_cfg.dev_id
    }

    pub fn set_failed(&mut self) {
        self.com_cfg.set_failed();
    }

    /// Returns the current status of the device, if VIRTIO_NET_F_STATUS
    /// has been negotiated. Otherwise returns zero.
    pub fn dev_status(&self) -> u16 {
        if self
            .dev_cfg
            .features
            .is_feature(Features::VIRTIO_NET_F_STATUS)
        {
            self.dev_cfg.raw.get_status()
        } else {
            0
        }
    }

    /// Returns the links status.
    /// If feature VIRTIO_NET_F_STATUS has not been negotiated, then we assume the link is up!
    pub fn is_link_up(&self) -> bool {
        if self
            .dev_cfg
            .features
            .is_feature(Features::VIRTIO_NET_F_STATUS)
        {
            self.dev_cfg.raw.get_status() & u16::from(Status::VIRTIO_NET_S_LINK_UP)
                == u16::from(Status::VIRTIO_NET_S_LINK_UP)
        } else {
            true
        }
    }

    #[allow(dead_code)]
    pub fn is_announce(&self) -> bool {
        if self
            .dev_cfg
            .features
            .is_feature(Features::VIRTIO_NET_F_STATUS)
        {
            self.dev_cfg.raw.get_status() & u16::from(Status::VIRTIO_NET_S_ANNOUNCE)
                == u16::from(Status::VIRTIO_NET_S_ANNOUNCE)
        } else {
            false
        }
    }

    /// Returns the maximal number of virtqueue pairs allowed. This is the
    /// dominant setting to define the number of virtqueues for the network
    /// device and overrides the num_vq field in the common config.
    ///
    /// Returns 1 (i.e. minimum number of pairs) if VIRTIO_NET_F_MQ is not set.
    #[allow(dead_code)]
    pub fn get_max_vq_pairs(&self) -> u16 {
        if self.dev_cfg.features.is_feature(Features::VIRTIO_NET_F_MQ) {
            self.dev_cfg.raw.get_max_virtqueue_pairs()
        } else {
            1
        }
    }

    pub fn disable_interrupts(&self) {
        // Für send und receive queues?
        // Nur für receive? Weil send eh ausgeschaltet ist?
        self.recv_vqs.disable_notifs();
    }

    pub fn enable_interrupts(&self) {
        // Für send und receive queues?
        // Nur für receive? Weil send eh ausgeschaltet ist?
        self.recv_vqs.enable_notifs();
    }

    /// Initiallizes the device in adherence to specification. Returns Some(VirtioNetError)
    /// upon failure and None in case everything worked as expected.
    ///
    /// See Virtio specification v1.1. - 3.1.1.
    ///                      and v1.1. - 5.1.5
    pub fn init_dev(&mut self) -> Result<(), VirtioNetError> {
        // Reset
        self.com_cfg.reset_dev();

        // Indiacte device, that OS noticed it
        self.com_cfg.ack_dev();

        // Indicate device, that driver is able to handle it
        self.com_cfg.set_drv();

        // Define minimal feature set
        let min_feats: Vec<Features> = vec![
            Features::VIRTIO_F_VERSION_1,
            Features::VIRTIO_NET_F_MAC,
            // Features::VIRTIO_NET_F_STATUS,
        ];

        let mut min_feat_set = FeatureSet::new(0);
        min_feat_set.set_features(&min_feats);
        let mut feats: Vec<Features> = min_feats;

        // If wanted, push new features into feats here:
        //
        // Indirect descriptors can be used
        // feats.push(Features::VIRTIO_F_RING_INDIRECT_DESC);
        // MTU setting can be used
        feats.push(Features::VIRTIO_NET_F_MTU);

        // Currently the driver does NOT support the features below.
        // In order to provide functionality for theses, the driver
        // needs to take care of calculating checksum in
        // RxQueues.post_processing()
        // feats.push(Features::VIRTIO_NET_F_GUEST_CSUM);
        // feats.push(Features::VIRTIO_NET_F_GUEST_TSO4);
        // feats.push(Features::VIRTIO_NET_F_GUEST_TSO6);

        // Negotiate features with device. Automatically reduces selected feats in order to meet device capabilities.
        // Aborts in case incompatible features are selected by the dricer or the device does not support min_feat_set.
        match self.negotiate_features(&feats) {
            Ok(_) => info!(
                "Driver found a subset of features for virtio device {:x}. Features are: {:?}",
                self.dev_cfg.dev_id, &feats
            ),
            Err(vnet_err) => {
                match vnet_err {
                    VirtioNetError::FeatReqNotMet(feat_set) => {
                        error!("Network drivers feature set {:x} does not satisfy rules in section 5.1.3.1 of specification v1.1. Aborting!", u64::from(feat_set));
                        return Err(vnet_err);
                    }
                    VirtioNetError::IncompFeatsSet(drv_feats, dev_feats) => {
                        // Create a new matching feature set for device and driver if the minimal set is met!
                        if (min_feat_set & dev_feats) != min_feat_set {
                            error!("Device features set, does not satisfy minimal features needed. Aborting!");
                            return Err(VirtioNetError::FailFeatureNeg(self.dev_cfg.dev_id));
                        } else {
                            feats = match Features::from_set(dev_feats & drv_feats) {
                                Some(feats) => feats,
                                None => {
                                    error!("Feature negotiation failed with minimal feature set. Aborting!");
                                    return Err(VirtioNetError::FailFeatureNeg(
                                        self.dev_cfg.dev_id,
                                    ));
                                }
                            };

                            match self.negotiate_features(&feats) {
                                Ok(_) => info!("Driver found a subset of features for virtio device {:x}. Features are: {:?}", self.dev_cfg.dev_id, &feats),
                                Err(vnet_err) => {
                                    match vnet_err {
                                        VirtioNetError::FeatReqNotMet(feat_set) => {
                                            error!("Network device offers a feature set {:x} when used completely does not satisfy rules in section 5.1.3.1 of specification v1.1. Aborting!", u64::from(feat_set));
                                            return Err(vnet_err);
                                        },
                                        _ => {
                                            error!("Feature Set after reduction still not usable. Set: {:?}. Aborting!", feats);
                                            return Err(vnet_err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        error!(
                            "Wanted set of features is NOT supported by device. Set: {:?}",
                            feats
                        );
                        return Err(vnet_err);
                    }
                }
            }
        }

        // Indicates the device, that the current feature set is final for the driver
        // and will not be changed.
        self.com_cfg.features_ok();

        // Checks if the device has accepted final set. This finishes feature negotiation.
        if self.com_cfg.check_features() {
            info!(
                "Features have been negotiated between virtio network device {:x} and driver.",
                self.dev_cfg.dev_id
            );
            // Set feature set in device config fur future use.
            self.dev_cfg.features.set_features(&feats);
        } else {
            return Err(VirtioNetError::FailFeatureNeg(self.dev_cfg.dev_id));
        }

        match self.dev_spec_init() {
            Ok(_) => info!(
                "Device specific initialization for Virtio network device {:x} finished",
                self.dev_cfg.dev_id
            ),
            Err(vnet_err) => return Err(vnet_err),
        }
        // At this point the device is "live"
        self.com_cfg.drv_ok();

        Ok(())
    }

    /// Negotiates a subset of features, understood and wanted by both the OS
    /// and the device.
    fn negotiate_features(&mut self, wanted_feats: &[Features]) -> Result<(), VirtioNetError> {
        let mut drv_feats = FeatureSet::new(0);

        for feat in wanted_feats.iter() {
            drv_feats |= *feat;
        }

        let dev_feats = FeatureSet::new(self.com_cfg.dev_features());

        // Checks if the selected feature set is compatible with requirements for
        // features according to Virtio spec. v1.1 - 5.1.3.1.
        match FeatureSet::check_features(wanted_feats) {
            Ok(_) => {
                info!("Feature set wanted by network driver are in conformance with specification.")
            }
            Err(vnet_err) => return Err(vnet_err),
        }

        if (dev_feats & drv_feats) == drv_feats {
            // If device supports subset of features write feature set to common config
            self.com_cfg.set_drv_features(drv_feats.into());
            Ok(())
        } else {
            Err(VirtioNetError::IncompFeatsSet(drv_feats, dev_feats))
        }
    }

    /// Device Specific initialization according to Virtio specifictation v1.1. - 5.1.5
    fn dev_spec_init(&mut self) -> Result<(), VirtioNetError> {
        match self.virtqueue_init() {
            Ok(_) => info!("Network driver successfully initialized virtqueues."),
            Err(vnet_err) => return Err(vnet_err),
        }

        // Add a control if feature is negotiated
        if self
            .dev_cfg
            .features
            .is_feature(Features::VIRTIO_NET_F_CTRL_VQ)
        {
            self.ctrl_vq = CtrlQueue(Some(Rc::new(Virtq::new(
                &mut self.com_cfg,
                &self.notif_cfg,
                VqSize::from(VIRTIO_MAX_QUEUE_SIZE),
                VqType::Split,
                VqIndex::from(self.num_vqs),
                self.dev_cfg.features.into(),
            ))));

            self.ctrl_vq.0.as_ref().unwrap().enable_notifs();
        }

        // If device does not take care of MAC address, the driver has to create one
        if !self.dev_cfg.features.is_feature(Features::VIRTIO_NET_F_MAC) {
            todo!("Driver created MAC address should be passed to device here.")
        }

        Ok(())
    }

    /// Initialize virtqueues via the queue interface and populates receiving queues
    fn virtqueue_init(&mut self) -> Result<(), VirtioNetError> {
        // We are assuming here, that the device single source of truth is the
        // device specific configuration. Hence we do NOT check if
        //
        // max_virtqueue_pairs + 1 < num_queues
        //
        // - the plus 1 is due to the possibility of an existing control queue
        // - the num_queues is found in the ComCfg struct of the device and defines the maximal number
        // of supported queues.
        if self.dev_cfg.features.is_feature(Features::VIRTIO_NET_F_MQ) {
            if self.dev_cfg.raw.get_max_virtqueue_pairs() * 2 >= MAX_NUM_VQ {
                self.num_vqs = MAX_NUM_VQ;
            } else {
                self.num_vqs = self.dev_cfg.raw.get_max_virtqueue_pairs() * 2;
            }
        } else {
            // Minimal number of virtqueues defined in the standard v1.1. - 5.1.5 Step 1
            self.num_vqs = 2;
        }

        // The loop is running from 0 to num_vqs and the indexes are provided to the VqIndex::from function in this way
        // in order to allow the indexes of the queues to be in a form of:
        //
        // index i for receiv queue
        // index i+1 for send queue
        //
        // as it is wanted by the network network device.
        // see Virtio specification v1.1. - 5.1.2
        // Assure that we have always an even number of queues (i.e. pairs of queues).
        assert_eq!(self.num_vqs % 2, 0);

        for i in 0..(self.num_vqs / 2) {
            let vq = Virtq::new(
                &mut self.com_cfg,
                &self.notif_cfg,
                VqSize::from(VIRTIO_MAX_QUEUE_SIZE),
                VqType::Split,
                VqIndex::from(2 * i),
                self.dev_cfg.features.into(),
            );
            // Interrupt for receiving packets is wanted
            vq.enable_notifs();

            self.recv_vqs.add(vq, &self.dev_cfg);

            let vq = Virtq::new(
                &mut self.com_cfg,
                &self.notif_cfg,
                VqSize::from(VIRTIO_MAX_QUEUE_SIZE),
                VqType::Split,
                VqIndex::from(2 * i + 1),
                self.dev_cfg.features.into(),
            );
            // Interrupt for comunicating that a sended packet left, is not needed
            vq.disable_notifs();

            self.send_vqs.add(vq, &self.dev_cfg);
        }

        Ok(())
    }
}

/// Error module of virtios network driver. Containing the (VirtioNetError)[VirtioNetError]
/// enum.
pub mod error {
    use crate::drivers::net::constants::FeatureSet;
    /// Network drivers error enum.
    #[derive(Debug, Copy, Clone)]
    pub enum VirtioNetError {
        General,
        NoDevCfg(u16),
        NoComCfg(u16),
        NoIsrCfg(u16),
        NoNotifCfg(u16),
        FailFeatureNeg(u16),
        /// Set of features does not adhere to the requirements of features
        /// indicated by the specification
        FeatReqNotMet(FeatureSet),
        /// The first u64 contains the feature bits wanted by the driver.
        /// but which are incompatible with the device feature set, second u64.
        IncompFeatsSet(FeatureSet, FeatureSet),
        /// Indicates that an operation for finished Transfers, was performed on
        /// an ongoing transfer
        ProcessOngoing,
        Unknown,
    }
}
