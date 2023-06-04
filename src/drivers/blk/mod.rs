pub mod constants;
pub mod mmio;
pub mod virtio_blk;

/// A trait for accessing the network interface
pub trait BlkInterface {
    /// Read blocks.
    fn read_block(&mut self, sector: usize, count: usize, buf: usize);
    /// Write blocks.
    fn write_block(&mut self, sector: usize, count: usize, buf: usize);
    /// Handle interrupt and check if a packet is available
    fn handle_interrupt(&mut self) -> bool;
}

#[cfg(feature = "fat")]
use crate::drivers::virtio::mmio::get_block_driver;

#[cfg(feature = "fat")]
pub fn blk_irqhandler() {
    match get_block_driver() {
        Some(driver) => {
            if !driver.lock().handle_interrupt() {
                error!("Virtio Blk driver failed to handler interrupt");
            }
        }
        _ => error!("failed to get block driver"),
    }
}

#[cfg(feature = "fat")]
pub fn read(sector: usize, count: usize, buf: usize) {
    match get_block_driver() {
        Some(driver) => driver.lock().read_block(sector, count, buf),
        _ => error!("failed to get block driver"),
    }
}

#[cfg(feature = "fat")]
pub fn write(sector: usize, count: usize, buf: usize) {
    match get_block_driver() {
        Some(driver) => driver.lock().write_block(sector, count, buf),
        _ => error!("failed to get block driver"),
    }
}
