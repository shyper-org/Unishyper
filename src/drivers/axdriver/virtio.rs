use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::{libs::traits::Address, mm::address::VAddr};
// use axalloc::global_allocator;
// use axhal::mem::{phys_to_virt, virt_to_phys};
use cfg_if::cfg_if;
use driver_common::{BaseDriverOps, DevResult, DeviceType};
use driver_virtio::{BufferDirection, PhysAddr, VirtIoHal};

use super::{drivers::DriverProbe, AxDeviceEnum};

cfg_if! {
    if #[cfg(feature = "pci")] {
        use driver_pci::{PciRoot, DeviceFunction, DeviceFunctionInfo};
        type VirtIoTransport = driver_virtio::PciTransport;
    } else {
        type VirtIoTransport = driver_virtio::MmioTransport;
    }
}

/// A trait for VirtIO device meta information.
pub trait VirtIoDevMeta {
    const DEVICE_TYPE: DeviceType;

    type Device: BaseDriverOps;
    type Driver = VirtIoDriver<Self>;

    fn try_new(transport: VirtIoTransport) -> DevResult<AxDeviceEnum>;
}

// cfg_if! {
//     if #[cfg(net_dev = "virtio-net")] {
pub struct VirtIoNet;

impl VirtIoDevMeta for VirtIoNet {
    const DEVICE_TYPE: DeviceType = DeviceType::Net;
    type Device = driver_virtio::VirtIoNetDev<VirtIoHalImpl, VirtIoTransport, 64>;

    fn try_new(transport: VirtIoTransport) -> DevResult<AxDeviceEnum> {
        Ok(AxDeviceEnum::from_net(Self::Device::try_new(transport)?))
    }
}
// }
// }

cfg_if! {
    if #[cfg(block_dev = "virtio-blk")] {
        pub struct VirtIoBlk;

        impl VirtIoDevMeta for VirtIoBlk {
            const DEVICE_TYPE: DeviceType = DeviceType::Block;
            type Device = driver_virtio::VirtIoBlkDev<VirtIoHalImpl, VirtIoTransport>;

            fn try_new(transport: VirtIoTransport) -> DevResult<AxDeviceEnum> {
                Ok(AxDeviceEnum::from_block(Self::Device::try_new(transport)?))
            }
        }
    }
}

cfg_if! {
    if #[cfg(display_dev = "virtio-gpu")] {
        pub struct VirtIoGpu;

        impl VirtIoDevMeta for VirtIoGpu {
            const DEVICE_TYPE: DeviceType = DeviceType::Display;
            type Device = driver_virtio::VirtIoGpuDev<VirtIoHalImpl, VirtIoTransport>;

            fn try_new(transport: VirtIoTransport) -> DevResult<AxDeviceEnum> {
                Ok(AxDeviceEnum::from_display(Self::Device::try_new(transport)?))
            }
        }
    }
}

/// A common driver for all VirtIO devices that implements [`DriverProbe`].
pub struct VirtIoDriver<D: VirtIoDevMeta + ?Sized>(PhantomData<D>);

impl<D: VirtIoDevMeta> DriverProbe for VirtIoDriver<D> {
    #[cfg(feature = "mmio")]
    fn probe_mmio(mmio_base: usize, mmio_size: usize) -> Option<AxDeviceEnum> {
        // let base_vaddr = phys_to_virt(mmio_base.into());
        let base_vaddr = mmio_base.pa2kva();
        if let Some((ty, transport)) =
            driver_virtio::probe_mmio_device(base_vaddr as *mut u8, mmio_size)
        {
            if ty == D::DEVICE_TYPE {
                match D::try_new(transport) {
                    Ok(dev) => return Some(dev),
                    Err(e) => {
                        warn!(
                            "failed to initialize MMIO device at [PA:{:#x}, PA:{:#x}): {:?}",
                            mmio_base,
                            mmio_base + mmio_size,
                            e
                        );
                        return None;
                    }
                }
            }
        }
        None
    }

    #[cfg(feature = "pci")]
    fn probe_pci(
        root: &mut PciRoot,
        bdf: DeviceFunction,
        dev_info: &DeviceFunctionInfo,
    ) -> Option<AxDeviceEnum> {
        if dev_info.vendor_id != 0x1af4 {
            return None;
        }
        match (D::DEVICE_TYPE, dev_info.device_id) {
            (DeviceType::Net, 0x1000) | (DeviceType::Net, 0x1040) => {}
            (DeviceType::Block, 0x1001) | (DeviceType::Block, 0x1041) => {}
            (DeviceType::Display, 0x1050) => {}
            _ => return None,
        }

        if let Some((ty, transport)) =
            driver_virtio::probe_pci_device::<VirtIoHalImpl>(root, bdf, dev_info)
        {
            if ty == D::DEVICE_TYPE {
                match D::try_new(transport) {
                    Ok(dev) => return Some(dev),
                    Err(e) => {
                        warn!(
                            "failed to initialize PCI device at {}({}): {:?}",
                            bdf, dev_info, e
                        );
                        return None;
                    }
                }
            }
        }
        None
    }
}

pub struct VirtIoHalImpl;

unsafe impl VirtIoHal for VirtIoHalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        debug!("dma_alloc pages {} dir {:?}", pages, _direction);

        let vaddr = if let Some(vaddr) = crate::mm::kallocate(pages * crate::arch::PAGE_SIZE) {
            vaddr.value()
        } else {
            return (0, NonNull::dangling());
        };
        // let vaddr = if let Ok(vaddr) = global_allocator().alloc_pages(pages, 0x1000) {
        //     vaddr
        // } else {
        //     return (0, NonNull::dangling());
        // };
        // let paddr = virt_to_phys(vaddr.into());
        let paddr = vaddr.kva2pa();
        let ptr = NonNull::new(vaddr as _).unwrap();
        (paddr, ptr)
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, _pages: usize) -> i32 {
        debug!(
            "dma_dealloc _paddr {:#x} vaddr {:#p} pages {}",
            _paddr,
            vaddr.as_ptr(),
            _pages
        );

        crate::mm::deallocate(VAddr::new_canonical(vaddr.as_ptr() as usize));
        // global_allocator().dealloc_pages(vaddr.as_ptr() as usize, pages);
        0
    }

    #[inline]
    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr.pa2kva() as *mut u8).unwrap()
    }

    #[inline]
    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        // virt_to_phys(vaddr.into()).into()
        vaddr.kva2pa().into()
    }

    #[inline]
    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}
