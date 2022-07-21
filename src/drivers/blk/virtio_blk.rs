use alloc::boxed::Box;
use core::mem::size_of;
use cortex_a::asm::barrier::*;

use spin::Mutex;
use tock_registers::interfaces::*;

use Operation::*;

use super::ring::*;
use super::blk::*;
use super::virtio::*;

use crate::drivers::virtio::mmio::MAGIC_VALUE;
use crate::drivers::virtio::transport::mmio::{DevId, MmioRegisterLayout};
use crate::lib::traits::Address;

const VIRTIO_MMIO_BASE: usize = 0x0a000000 | 0xFFFF_FF80_0000_0000;
const QUEUE_SIZE: usize = 8;
const VIRTIO_F_VERSION_1: u32 = 32;

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
struct VirtioRing {
    desc: [VirtioRingDesc; QUEUE_SIZE],
    driver: VirtioRingDriver,
    device: VirtioRingDevice,
}

static VIRTIO_RING: Mutex<VirtioRing> = Mutex::new(VirtioRing {
    desc: [VirtioRingDesc {
        addr: 0,
        len: 0,
        flags: 0,
        next: 0,
    }; QUEUE_SIZE],
    driver: VirtioRingDriver {
        flags: 0,
        idx: 0,
        ring: [0; QUEUE_SIZE],
    },
    device: VirtioRingDevice {
        flags: 0,
        idx: 0,
        ring: [VirtioRingDeviceElement { id: 0, len: 0 }; QUEUE_SIZE],
    },
});

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct VirtioRingDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
#[derive(Debug)]
struct VirtioRingDriver {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_SIZE],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct VirtioRingDeviceElement {
    id: u32,
    len: u32,
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
struct VirtioRingDevice {
    flags: u16,
    idx: u16,
    ring: [VirtioRingDeviceElement; QUEUE_SIZE],
}

struct VirtioMmio {
    base_addr: usize,
}

impl core::ops::Deref for VirtioMmio {
    type Target = MmioRegisterLayout;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl VirtioMmio {
    const fn new(base_addr: usize) -> Self {
        VirtioMmio { base_addr }
    }
    fn ptr(&self) -> *const MmioRegisterLayout {
        self.base_addr as *const _
    }
}

trait BaseAddr {
    fn base_addr_u64(&self) -> u64;
    fn base_addr_usize(&self) -> usize;
}

impl<T> BaseAddr for T {
    fn base_addr_u64(&self) -> u64 {
        (self as *const T as usize).kva2pa() as u64
    }
    fn base_addr_usize(&self) -> usize {
        (self as *const T as usize).kva2pa() as usize
    }
}

static VIRTIO_MMIO: VirtioMmio = VirtioMmio::new(VIRTIO_MMIO_BASE);

fn virtio_mmio_setup_vq(index: usize) {
    let index = index as u32;
    let mmio = &VIRTIO_MMIO;
    let num = mmio.get_max_queue_size(index);
    if num == 0 {
        panic!("queue num max is zero");
    } else if num < QUEUE_SIZE as u32 {
        panic!("queue size not supported");
    }
    mmio.set_queue_size(index, QUEUE_SIZE as u32);

    let ring = VIRTIO_RING.lock();

    mmio.set_ring_addr(index, ring.desc.base_addr_usize());
    mmio.set_drv_ctrl_addr(index, ring.driver.base_addr_usize());
    mmio.set_dev_ctrl_addr(index, ring.device.base_addr_usize());

    mmio.enable_queue(index);
}

pub fn virtio_blk_init() {
    let mmio = &VIRTIO_MMIO;
    if mmio.get_magic_value() != MAGIC_VALUE
        || mmio.get_version() != 2
        || mmio.get_device_id() != DevId::VIRTIO_DEV_ID_BLK
        || mmio.get_vendor_id() != 0x554d4551
    {
        panic!("could not find virtio blk")
    }

    let mut status = VIRTIO_CONFIG_S_ACKNOWLEDGE as u32;
    mmio.status.set(status);
    status |= VIRTIO_CONFIG_S_DRIVER as u32;
    mmio.status.set(status);

    let feature: u64 = 1 << VIRTIO_F_VERSION_1
        | 1 << VIRTIO_BLK_F_SEG_MAX
        | 1 << VIRTIO_BLK_F_GEOMETRY
        | 1 << VIRTIO_BLK_F_BLK_SIZE
        | 1 << VIRTIO_BLK_F_TOPOLOGY;

    mmio.set_drv_features(feature);

    status |= VIRTIO_CONFIG_S_FEATURES_OK as u32;
    mmio.status.set(status);

    status |= VIRTIO_CONFIG_S_DRIVER_OK as u32;
    mmio.status.set(status);

    virtio_mmio_setup_vq(0);
    info!("virtio_blk_init OK");
}

pub enum Operation {
    Read,
    Write,
}

#[repr(C)]
pub struct VirtioBlkOutHdr {
    t: u32,
    priority: u32,
    sector: u64,
}

pub fn read(sector: usize, count: usize, buf: usize) {
    io(sector, count, buf, Read);
}

pub fn write(sector: usize, count: usize, buf: usize) /* -> Box<DiskRequest>*/
{
    io(sector, count, buf, Write);
}

fn io(sector: usize, count: usize, buf: usize, op: Operation) {
    let hdr = Box::new(VirtioBlkOutHdr {
        t: match op {
            Operation::Read => 0,
            Operation::Write => 1,
        },
        priority: 0,
        sector: sector as u64,
        // status: 255,
    });

    let status = Box::new(255u8);
    let mut ring = VIRTIO_RING.lock();

    let desc = ring.desc.get_mut(0).unwrap();
    desc.addr = (hdr.as_ref() as *const VirtioBlkOutHdr as usize).kva2pa() as u64;
    desc.len = size_of::<VirtioBlkOutHdr>() as u32;
    desc.flags = VRING_DESC_F_NEXT;
    desc.next = 1;

    let desc = ring.desc.get_mut(1).unwrap();
    desc.addr = buf.kva2pa() as u64;
    desc.len = (512 * count) as u32;
    desc.flags = match op {
        Operation::Read => VRING_DESC_F_WRITE,
        Operation::Write => 0,
    };
    desc.flags |= VRING_DESC_F_NEXT;
    desc.next = 2;

    let desc = ring.desc.get_mut(2).unwrap();
    desc.addr = (status.as_ref() as *const u8 as usize).kva2pa() as u64;
    desc.len = 1;
    desc.flags = VRING_DESC_F_WRITE;
    desc.next = 0;

    // avail[0] is flags
    // avail[1] tells the device how far to look in avail[2...].
    // avail[2...] are desc[] indices the device should process.
    // we only tell device the first index in our chain of descriptors.
    let avail = &mut ring.driver;
    avail.ring[(avail.idx as usize) % QUEUE_SIZE] = 0;
    unsafe {
        dsb(SY);
    }
    avail.idx = avail.idx.wrapping_add(1);

    let mmio = &VIRTIO_MMIO;

    mmio.queue_notify.set(0); // queue num

    irq(status);
}

fn irq(status: Box<u8>) {
    loop {
        unsafe {
            dsb(SY);
        }
        if *status == VIRTIO_BLK_S_OK {
            return;
        } else if *status == VIRTIO_BLK_S_IOERR {
            panic!("VIRTIO_BLK_S_IOERR");
        } else if *status == VIRTIO_BLK_S_UNSUPP {
            panic!("VIRTIO_BLK_S_UNSUPP");
        } else if *status == 255 {
            continue;
        }
        // if mmio.InterruptStatus.get() == 1 {
        //     mmio.InterruptACK.set(1);
        //     break;
        // }
    }
}
