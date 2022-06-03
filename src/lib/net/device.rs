// use alloc::collections::btree_map::BTreeMap;
use alloc::vec;
use core::slice;

#[cfg(not(feature = "dhcpv4"))]
use no_std_net::Ipv4Addr;

#[cfg(feature = "dhcpv4")]
use smoltcp::dhcp::Dhcpv4Client;
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache, Routes};
#[cfg(feature = "trace")]
use smoltcp::phy::EthernetTracer;
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

extern "Rust" {
    fn sys_get_mac_address() -> Result<[u8; 6], ()>;
    fn sys_get_mtu() -> Result<u16, ()>;
    fn sys_get_tx_buffer(len: usize) -> Result<(*mut u8, usize), ()>;
    fn sys_send_tx_buffer(handle: usize, len: usize) -> Result<(), ()>;
    fn sys_receive_rx_buffer() -> Result<(&'static mut [u8], usize), ()>;
    fn sys_rx_buffer_consumed(handle: usize) -> Result<(), ()>;
    fn sys_free_tx_buffer(handle: usize);
}

/// Data type to determine the mac address
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct ShyperNet {
    pub mtu: u16,
}

impl ShyperNet {
    pub(crate) const fn new(mtu: u16) -> Self {
        Self { mtu }
    }
}

impl NetworkInterface<ShyperNet> {
    #[cfg(feature = "dhcpv4")]
    pub(crate) fn new() -> NetworkState {
        let mtu = match unsafe { sys_get_mtu() } {
            Ok(mtu) => mtu,
            Err(_) => {
                return NetworkState::InitializationFailed;
            }
        };
        let device = ShyperNet::new(mtu);
        #[cfg(feature = "trace")]
        let device = EthernetTracer::new(device, |_timestamp, printer| {
            trace!("{}", printer);
        });

        let mac: [u8; 6] = match unsafe { sys_get_mac_address() } {
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
    pub(crate) fn new() -> NetworkState {
        let mtu = match unsafe { sys_get_mtu() } {
            Ok(mtu) => mtu,
            Err(_) => {
                return NetworkState::InitializationFailed;
            }
        };
        let device = ShyperNet::new(mtu);
        #[cfg(feature = "trace")]
        let device = EthernetTracer::new(device, |_timestamp, printer| {
            trace!("{}", printer);
        });

        let mac: [u8; 6] = match unsafe { sys_get_mac_address() } {
            Ok(mac) => mac,
            Err(_) => {
                return NetworkState::InitializationFailed;
            }
        };

        let myip = Ipv4Addr::new(10, 0, 5, 3);
        let myip = myip.octets();
        let mygw = Ipv4Addr::new(10, 0, 5, 1);
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

		// How to use it with Heap?
        // let neighbor_cache = NeighborCache::new(BTreeMap::new());
        let mut neighbor_cache_storage = [None; 8];
        let mut neighbor_cache = NeighborCache::new(&mut neighbor_cache_storage[..]);
        let ethernet_addr = EthernetAddress([mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]]);
        let ip_addrs = [IpCidr::new(
            IpAddress::v4(myip[0], myip[1], myip[2], myip[3]),
            prefix_len.try_into().unwrap(),
        )];
        let default_v4_gw = Ipv4Address::new(mygw[0], mygw[1], mygw[2], mygw[3]);
        // let mut routes = Routes::new(BTreeMap::new());
		let mut routes_storage = [];
	let mut routes = Routes::new(&mut routes_storage[..]);
        routes.add_default_ipv4_route(default_v4_gw).unwrap();

        info!("MAC address {}", ethernet_addr);
        info!("Configure network interface with address {}", ip_addrs[0]);
        info!("Configure gateway with address {}", default_v4_gw);
        info!("MTU: {} bytes", mtu);

        let iface = EthernetInterfaceBuilder::new(device)
            .ethernet_addr(ethernet_addr)
            .neighbor_cache(neighbor_cache)
            .ip_addrs(ip_addrs.as_mut_slice())
            .routes(routes)
            .finalize();

        NetworkState::Initialized(Self {
            iface,
            sockets: SocketSet::new(vec![].as_mut_slice()),
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
        match unsafe { sys_receive_rx_buffer() } {
            Ok((buffer, handle)) => Some((RxToken::new(buffer, handle), TxToken::new())),
            _ => None,
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        trace!("create TxToken to transfer data");
        Some(TxToken::new())
    }
}

#[doc(hidden)]
pub(crate) struct RxToken {
    buffer: &'static mut [u8],
    handle: usize,
}

impl RxToken {
    pub(crate) fn new(buffer: &'static mut [u8], handle: usize) -> Self {
        Self { buffer, handle }
    }
}

impl phy::RxToken for RxToken {
    #[allow(unused_mut)]
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let result = f(self.buffer);
        if unsafe { sys_rx_buffer_consumed(self.handle).is_ok() } {
            result
        } else {
            Err(smoltcp::Error::Exhausted)
        }
    }
}

#[doc(hidden)]
pub(crate) struct TxToken;

impl TxToken {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(self, _timestamp: Instant, len: usize, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let (tx_buffer, handle) =
            unsafe { sys_get_tx_buffer(len).map_err(|_| smoltcp::Error::Exhausted)? };
        let tx_slice: &'static mut [u8] = unsafe { slice::from_raw_parts_mut(tx_buffer, len) };
        match f(tx_slice) {
            Ok(result) => {
                if unsafe { sys_send_tx_buffer(handle, len).is_ok() } {
                    Ok(result)
                } else {
                    Err(smoltcp::Error::Exhausted)
                }
            }
            Err(e) => {
                unsafe { sys_free_tx_buffer(handle) };
                Err(e)
            }
        }
    }
}