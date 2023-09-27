use alloc::vec;
use alloc::boxed::Box;
use core::slice;

use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress};
use smoltcp::wire::{IpAddress, IpCidr};

use super::interface::{NetworkInterface, NetworkState};

use crate::drivers::get_network_driver;
// use crate::drivers::net::{
//     get_mac_address, get_mtu, get_tx_buffer, send_tx_buffer, receive_rx_buffer, rx_buffer_consumed,
//     free_tx_buffer,
// };

const STANDARD_MTU: usize = 1500;

const RANDOM_SEED: u64 = 0xA2CE_05A2_CE05_A2CE;
const IP: &str = "10.0.0.2";
const GATEWAY: &str = "10.0.0.1";
const IP_PREFIX: u8 = 24;

pub struct ShyperNet {
    // mtu: u16,
    // with_checksums: bool,
}

impl ShyperNet {
    pub const fn new() -> Self {
        Self {
            // mtu,
            // with_checksums,
        }
    }
}

impl<'a> NetworkInterface<'a> {
    pub fn new() -> NetworkState<'a> {
        let mac = if let Some(driver) = get_network_driver() {
            driver.lock().get_mac_address()
        } else {
            return NetworkState::InitializationFailed;
        };

        let mut device = ShyperNet::new();

        let myip = IP.parse::<IpAddress>().expect("invalid IP address");
        let mygw = GATEWAY
            .parse::<IpAddress>()
            .expect("invalid gateway IP address");

        let ethernet_addr = EthernetAddress(mac);
        let hardware_addr = HardwareAddress::Ethernet(ethernet_addr);
        let ip_addrs = [IpCidr::new(myip, IP_PREFIX)];

        info!("MAC address {}", hardware_addr);
        info!("Configure network interface with address {}", ip_addrs[0]);
        info!("Configure gateway with address {}", mygw);

        // use the current time based on the wall-clock time as seed
        let mut config = Config::new(hardware_addr);

        config.random_seed = RANDOM_SEED;
        if device.capabilities().medium == Medium::Ethernet {
            config.hardware_addr = hardware_addr;
        }

        let mut iface = Interface::new(config, &mut device, super::now());
        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs.push(IpCidr::new(myip, IP_PREFIX)).unwrap();
        });
        match mygw {
            IpAddress::Ipv4(v4) => iface.routes_mut().add_default_ipv4_route(v4).unwrap(),
            IpAddress::Ipv6(v6) => unimplemented!("Unsuppported Ipv6 gateway {:?}", v6),
        };

        NetworkState::Initialized(Box::new(Self {
            iface,
            sockets: SocketSet::new(vec![]),
            device,
        }))
    }
}

impl Device for ShyperNet {
    type RxToken<'a> = RxToken;
    type TxToken<'a> = TxToken;

    /// Get a description of device capabilities.
    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.max_transmission_unit = STANDARD_MTU + 14;
        cap
    }

    /// Construct a token pair consisting of one receive token and one transmit token.
    ///
    /// The additional transmit token makes it possible to generate a reply packet based
    /// on the contents of the received packet. For example, this makes it possible to
    /// handle arbitrarily large ICMP echo ("ping") requests, where the all received bytes
    /// need to be sent back, without heap allocation.
    ///
    /// The timestamp must be a number of milliseconds, monotonically increasing since an
    /// arbitrary moment in time, such as system startup.
    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        // trace!("ShyperNet receive receive_rx_buffer()");
        match get_network_driver().unwrap().lock().receive_rx_buffer() {
            Ok((buffer, handle)) => Some((RxToken::new(buffer, handle), TxToken::new())),
            _ => None,
        }
    }

    /// Construct a transmit token.
    ///
    /// The timestamp must be a number of milliseconds, monotonically increasing since an
    /// arbitrary moment in time, such as system startup.
    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        // debug!("create TxToken to transfer data");
        Some(TxToken::new())
    }
}

pub struct RxToken {
    buffer: &'static mut [u8],
    handle: usize,
}

/// A token to receive a single network packet.
impl RxToken {
    pub fn new(buffer: &'static mut [u8], handle: usize) -> Self {
        Self { buffer, handle }
    }
}

impl phy::RxToken for RxToken {
    /// Consumes the token to receive a single network packet.
    ///
    /// This method receives a packet and then calls the given closure `f` with the raw
    /// packet bytes as argument.
    ///
    /// The timestamp must be a number of milliseconds, monotonically increasing since an
    /// arbitrary moment in time, such as system startup.
    #[allow(unused_mut)]
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut rx_buf = self.buffer;
        trace!("RECV {} bytes: {:02X?}", rx_buf.len(), rx_buf);
        let result = f(rx_buf);
        get_network_driver()
            .unwrap()
            .lock()
            .rx_buffer_consumed(self.handle);
        result
    }
}

pub struct TxToken;

/// A token to transmit a single network packet.
impl TxToken {
    pub fn new() -> Self {
        Self {}
    }
}

impl phy::TxToken for TxToken {
    /// Consumes the token to send a single network packet.
    ///
    /// This method constructs a transmit buffer of size `len` and calls the passed
    /// closure `f` with a mutable reference to that buffer. The closure should construct
    /// a valid network packet (e.g. an ethernet packet) in the buffer. When the closure
    /// returns, the transmit buffer is sent out.
    ///
    /// The timestamp must be a number of milliseconds, monotonically increasing since an
    /// arbitrary moment in time, such as system startup.
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut dev = get_network_driver().unwrap().lock();
        let (tx_buffer, handle) = dev
            .get_tx_buffer(len)
            .expect("TxToken: failed in get_tx_buffer");
        let tx_slice: &'static mut [u8] = unsafe { slice::from_raw_parts_mut(tx_buffer, len) };
        let ret = f(tx_slice);

        trace!("SEND {} bytes: {:02X?}", len, tx_slice);
        match dev.send_tx_buffer(handle, len) {
            Ok(()) => {}
            Err(_) => {
                warn!("TxToken consume error");
            }
        }

        ret
    }
}
