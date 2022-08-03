pub mod constants;
pub mod mmio;
pub mod virtio_blk;

#[cfg(feature = "oldfs")]
mod virtio_blk_ori;

/// A trait for accessing the network interface
pub trait BlkInterface {
    /// Read blocks.
    fn read_block(&mut self, sector: usize, count: usize, buf: usize);
    /// Write blocks.
    fn write_block(&mut self, sector: usize, count: usize, buf: usize);
    /// Handle interrupt and check if a packet is available
    fn handle_interrupt(&mut self) -> bool;
}

use crate::drivers::virtio::mmio::get_block_driver;
pub fn blk_irqhandler() {
    #[cfg(feature = "fs")]
    match get_block_driver() {
        Some(driver) => {
            if !driver.lock().handle_interrupt() {
                error!("Virtio Blk driver failed to handler interrupt");
            }
        },
        _ => error!("failed to get block driver"),
    }
}

#[cfg(feature = "oldfs")]
pub fn virtio_blk_init() {
    virtio_blk_ori::virtio_blk_init();
}

pub fn read(sector: usize, count: usize, buf: usize) {
    #[cfg(feature = "oldfs")]
    virtio_blk_ori::read(sector, count, buf);
    #[cfg(feature = "fs")]
    match get_block_driver() {
        Some(driver) => driver.lock().read_block(sector, count, buf),
        _ => error!("failed to get block driver"),
    }
}

pub fn write(sector: usize, count: usize, buf: usize) {
    #[cfg(feature = "oldfs")]
    virtio_blk_ori::write(sector, count, buf);
    #[cfg(feature = "fs")]
    match get_block_driver() {
        Some(driver) => driver.lock().write_block(sector, count, buf),
        _ => error!("failed to get block driver"),
    }
}
