// use alloc::collections::btree_map::BTreeMap;
use alloc::vec;
use core::slice;
use alloc::collections::BTreeMap;

#[cfg(not(feature = "dhcpv4"))]
use no_std_net::Ipv4Addr;

#[cfg(feature = "dhcpv4")]
use smoltcp::dhcp::Dhcpv4Client;
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache, Routes};
use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::socket::SocketSet;
#[cfg(feature = "dhcpv4")]
use smoltcp::socket::{RawPacketMetadata, RawSocketBuffer};
use smoltcp::time::Instant;
#[cfg(not(feature = "dhcpv4"))]
use smoltcp::wire::IpAddress;
#[cfg(feature = "dhcpv4")]
use smoltcp::wire::Ipv4Cidr;
use smoltcp::wire::{EthernetAddress, IpCidr, Ipv4Address};

use super::interface::{NetworkInterface, NetworkState};
use super::waker::WakerRegistration;

use crate::drivers::net::{
    get_mac_address,
    get_mtu,
    get_tx_buffer,
    send_tx_buffer,
    receive_rx_buffer,
    rx_buffer_consumed,
    free_tx_buffer,
};

/// Data type to determine the mac address
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ShyperNet {
    pub mtu: u16,
}

impl ShyperNet {
    pub const fn new(mtu: u16) -> Self {
        Self { mtu }
    }
}

impl NetworkInterface<ShyperNet> {
    #[cfg(feature = "dhcpv4")]
    pub fn new() -> NetworkState {
        let mtu = match get_mtu(){
            Ok(mtu) => mtu,
            Err(_) => {
                return NetworkState::InitializationFailed;
            }
        };
        let device = ShyperNet::new(mtu);

        let mac: [u8; 6] = match get_mac_address() {
            Ok(mac) => mac,
            Err(_) => {
                return NetworkState::InitializationFailed;
            }
        };

        let neighbor_cache = NeighborCache::new(BTreeMap::new());
        let ethernet_addr = EthernetAddress([mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]]);
        let ip_addrs = [IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0)];
        let routes = Routes::new(BTreeMap::new());

        info!("MAC address {}", ethernet_addr);
        info!("MTU: {} bytes", mtu);

        let mut sockets = SocketSet::new(vec![]);
        let dhcp_rx_buffer = RawSocketBuffer::new([RawPacketMetadata::EMPTY; 1], vec![0; 900]);
        let dhcp_tx_buffer = RawSocketBuffer::new([RawPacketMetadata::EMPTY; 1], vec![0; 600]);
        let dhcp = Dhcpv4Client::new(
            &mut sockets,
            dhcp_rx_buffer,
            dhcp_tx_buffer,
            Instant::from_millis(current_ms() as i64),
        );
        let prev_cidr = Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0);

        let iface = EthernetInterfaceBuilder::new(device)
            .ethernet_addr(ethernet_addr)
            .neighbor_cache(neighbor_cache)
            .ip_addrs(ip_addrs)
            .routes(routes)
            .finalize();

        NetworkState::Initialized(Self {
            iface,
            sockets,
            dhcp,
            prev_cidr,
            waker: WakerRegistration::new(),
        })
    }

    #[cfg(not(feature = "dhcpv4"))]
    pub fn new() -> NetworkState {
        info!("Network interface new:");
        // Get mtu, Maximum transmission unit.
        let mtu = match get_mtu() {
            Ok(mtu) => mtu,
            Err(_) => {
                return NetworkState::InitializationFailed;
            }
        };
        // New physical device, ShyperNet.
        let device = ShyperNet::new(mtu);

        // Get mac address.
        let mac: [u8; 6] = match get_mac_address() {
            Ok(mac) => mac,
            Err(_) => {
                return NetworkState::InitializationFailed;
            }
        };

        // Generate local ip address ,gateway address and network mask.
        let myip = Ipv4Addr::new(10, 0, 0, 2);
        let myip = myip.octets();
        let mygw = Ipv4Addr::new(10, 0, 0, 1);
        let mygw = mygw.octets();
        let mymask = Ipv4Addr::new(255, 255, 255, 0);
        let mymask = mymask.octets();

        // calculate the netmask length
        // => count the number of contiguous 1 bits,
        // starting at the most significant bit in the first octet

        let mut prefix_len = (!mymask[0]).trailing_zeros();
        if prefix_len == 8 {
            prefix_len += (!mymask[1]).trailing_zeros();
        }
        if prefix_len == 16 {
            prefix_len += (!mymask[2]).trailing_zeros();
        }
        if prefix_len == 24 {
            prefix_len += (!mymask[3]).trailing_zeros();
        }

        let neighbor_cache = NeighborCache::new(BTreeMap::new());
        let ethernet_addr = EthernetAddress([mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]]);
        let ip_addrs = [IpCidr::new(
            IpAddress::v4(myip[0], myip[1], myip[2], myip[3]),
            prefix_len.try_into().unwrap(),
        )];

        let default_v4_gw = Ipv4Address::new(mygw[0], mygw[1], mygw[2], mygw[3]);
        let mut routes = Routes::new(BTreeMap::new());
        routes.add_default_ipv4_route(default_v4_gw).unwrap();

        info!("MAC address {}", ethernet_addr);
        info!("Configure network interface with address {}", ip_addrs[0]);
        info!("Configure gateway with address {}", default_v4_gw);
        info!("MTU: {} bytes", mtu);

        let iface = EthernetInterfaceBuilder::new(device)
            .ethernet_addr(ethernet_addr)
            .neighbor_cache(neighbor_cache)
            .ip_addrs(ip_addrs)
            .routes(routes)
            .finalize();

        NetworkState::Initialized(Self {
            iface,
            sockets: SocketSet::new(vec![]),
            waker: WakerRegistration::new(),
        })
    }
}

impl<'a> Device<'a> for ShyperNet {
    type RxToken = RxToken;
    type TxToken = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.max_transmission_unit = self.mtu.into();
        cap
    }

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        // trace!("receive_rx_buffer()");
        match receive_rx_buffer() {
            Ok((buffer, handle)) => Some((RxToken::new(buffer, handle), TxToken::new())),
            _ => None,
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        // debug!("create TxToken to transfer data");
        Some(TxToken::new())
    }
}

#[doc(hidden)]
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
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let result = f(self.buffer);
        if rx_buffer_consumed(self.handle).is_ok() {
            result
        } else {
            Err(smoltcp::Error::Exhausted)
        }
    }
}

#[doc(hidden)]
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
    fn consume<R, F>(self, _timestamp: Instant, len: usize, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let (tx_buffer, handle) =
            get_tx_buffer(len).map_err(|_| smoltcp::Error::Exhausted)?;
        let tx_slice: &'static mut [u8] = unsafe { slice::from_raw_parts_mut(tx_buffer, len) };
        match f(tx_slice) {
            Ok(result) => {
                if send_tx_buffer(handle, len).is_ok(){
                    Ok(result)
                } else {
                    Err(smoltcp::Error::Exhausted)
                }
            }
            Err(e) => {
                _ = free_tx_buffer(handle);
                Err(e)
            }
        }
    }
}
