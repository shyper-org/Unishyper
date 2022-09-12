pub mod constants;
pub mod virtio_mmio;
pub mod virtio_net;

/// A trait for accessing the network interface
pub trait NetworkInterface {
    /// Returns the mac address of the device.
    fn get_mac_address(&self) -> [u8; 6];
    /// Returns the current MTU of the device.
    fn get_mtu(&self) -> u16;
    /// Get buffer to create a TX packet
    /// This returns ownership of the TX buffer.
    fn get_tx_buffer(&mut self, len: usize) -> Result<(*mut u8, usize), ()>;
    /// Frees the TX buffer (takes ownership)
    fn free_tx_buffer(&self, token: usize);
    /// Send TC packets (takes TX buffer ownership)
    fn send_tx_buffer(&mut self, tkn_handle: usize, len: usize) -> Result<(), ()>;
    /// Check if a packet is available
    fn has_packet(&self) -> bool;
    /// Get RX buffer with an received packet
    fn receive_rx_buffer(&mut self) -> Result<(&'static mut [u8], usize), ()>;
    /// Tells driver, that buffer is consumed and can be deallocated
    fn rx_buffer_consumed(&mut self, trf_handle: usize);
    /// Enable / disable the polling mode of the network interface
    fn set_polling_mode(&mut self, value: bool);
    /// Handle interrupt and check if a packet is available
    fn handle_interrupt(&mut self) -> bool;
}

use crate::drivers::virtio::mmio::get_network_driver;
use crate::lib::synch::{semaphore::Semaphore, spinlock::SpinlockIrqSave};
use crate::util::irqsave;

static NET_SEM: Semaphore = Semaphore::new(0);

pub extern "C" fn netwait() {
    irqsave(|| {
        trace!("netwait");
        NET_SEM.acquire();
        trace!("netwait acquire, return");
    });
}

#[no_mangle]
pub fn netwakeup() {
    irqsave(|| {
        trace!("netwakeup");
        NET_SEM.release();
    });
}

pub fn network_irqhandler() {
    debug!("Receive network interrupt");

    let check_scheduler = match get_network_driver() {
        Some(driver) => driver.lock().handle_interrupt(),
        _ => {
            error!("Unable to handle interrupt!");
            false
        }
    };

    if check_scheduler {
        crate::lib::thread::thread_schedule();
    }
}

/// set driver in polling mode and threads will not be blocked
pub extern "C" fn set_polling_mode(value: bool) {
    let icntr = crate::lib::timer::current_cycle();
    static THREADS_IN_POLLING_MODE: SpinlockIrqSave<usize> = SpinlockIrqSave::new(0);
    irqsave(|| {
        let mut guard = THREADS_IN_POLLING_MODE.lock();

        if value {
            *guard += 1;

            if *guard == 1 {
                if let Some(driver) = get_network_driver() {
                    driver.lock().set_polling_mode(value)
                }
            }
        } else {
            *guard -= 1;

            if *guard == 0 {
                if let Some(driver) = get_network_driver() {
                    driver.lock().set_polling_mode(value)
                }
            }
        }
    });
    let icntr2 = crate::lib::timer::current_cycle();
    debug!("net set_polling_mode {} cycle {}", value, icntr2 - icntr);
}

pub fn get_mac_address() -> Result<[u8; 6], ()> {
    match get_network_driver() {
        Some(driver) => Ok(driver.lock().get_mac_address()),
        _ => Err(()),
    }
}

pub fn get_mtu() -> Result<u16, ()> {
    match get_network_driver() {
        Some(driver) => Ok(driver.lock().get_mtu()),
        _ => Err(()),
    }
}

pub fn get_tx_buffer(len: usize) -> Result<(*mut u8, usize), ()> {
    match get_network_driver() {
        Some(driver) => driver.lock().get_tx_buffer(len),
        _ => Err(()),
    }
}

pub fn free_tx_buffer(handle: usize) -> Result<(), ()> {
    match get_network_driver() {
        Some(driver) => {
            driver.lock().free_tx_buffer(handle);
            Ok(())
        }
        _ => Err(()),
    }
}

pub fn send_tx_buffer(handle: usize, len: usize) -> Result<(), ()> {
    match get_network_driver() {
        Some(driver) => driver.lock().send_tx_buffer(handle, len),
        _ => Err(()),
    }
}

pub fn receive_rx_buffer() -> Result<(&'static mut [u8], usize), ()> {
    match get_network_driver() {
        Some(driver) => driver.lock().receive_rx_buffer(),
        _ => Err(()),
    }
}

pub fn rx_buffer_consumed(handle: usize) -> Result<(), ()> {
    match get_network_driver() {
        Some(driver) => {
            driver.lock().rx_buffer_consumed(handle);
            Ok(())
        }
        _ => Err(()),
    }
}
