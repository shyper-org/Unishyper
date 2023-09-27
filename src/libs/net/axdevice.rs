use alloc::vec;
use alloc::boxed::Box;

use driver_net::{DevError, NetBufPtr};

use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress};
use smoltcp::wire::{IpAddress, IpCidr};

use super::interface::{NetworkInterface, NetworkState};

use crate::drivers::get_network_driver;

const STANDARD_MTU: usize = 1500;

const RANDOM_SEED: u64 = 0xA2CE_05A2_CE05_A2CE;
const IP: &str = "10.0.0.2";
const GATEWAY: &str = "10.0.0.1";
const IP_PREFIX: u8 = 24;

pub struct ShyperNet {}

impl ShyperNet {
    pub const fn new() -> Self {
        Self {}
    }
}

impl<'a> NetworkInterface<'a> {
    pub fn new() -> NetworkState<'a> {
        let mac = if let Some(driver) = get_network_driver() {
            driver.lock().mac_address().0
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
    type RxToken<'a> = AxNetRxToken;
    type TxToken<'a> = AxNetTxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        // Ethernet MTU = IP MTU + 14.
        caps.max_transmission_unit = STANDARD_MTU + 14;
        caps.max_burst_size = None;
        caps.medium = Medium::Ethernet;
        caps
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut dev = get_network_driver().unwrap().lock();
        if let Err(e) = dev.recycle_tx_buffers() {
            warn!("recycle_tx_buffers failed: {:?}", e);
            return None;
        }

        if !dev.can_transmit() {
            return None;
        }
        let rx_buf = match dev.receive() {
            Ok(buf) => buf,
            Err(err) => {
                if !matches!(err, DevError::Again) {
                    warn!("receive failed: {:?}", err);
                }
                return None;
            }
        };
        Some((AxNetRxToken(rx_buf), AxNetTxToken()))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        let mut dev = get_network_driver().unwrap().lock();
        if let Err(e) = dev.recycle_tx_buffers() {
            warn!("recycle_tx_buffers failed: {:?}", e);
            return None;
        }
        if dev.can_transmit() {
            Some(AxNetTxToken())
        } else {
            None
        }
    }
}

pub struct AxNetRxToken(NetBufPtr);

pub struct AxNetTxToken();

impl phy::RxToken for AxNetRxToken {
    // fn preprocess(&self, sockets: &mut SocketSet<'_>) {
    //     snoop_tcp_packet(self.1.packet(), sockets).ok();
    // }

    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut rx_buf = self.0;
        trace!(
            "RECV {} bytes: {:02X?}",
            rx_buf.packet_len(),
            rx_buf.packet()
        );
        let result = f(rx_buf.packet_mut());
        let mut dev = get_network_driver().unwrap().lock();
        dev.recycle_rx_buffer(rx_buf).unwrap();
        result
    }
}

impl phy::TxToken for AxNetTxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut dev = get_network_driver().unwrap().lock();
        let mut tx_buf = dev.alloc_tx_buffer(len).unwrap();
        let ret = f(tx_buf.packet_mut());
        trace!("SEND {} bytes: {:02X?}", len, tx_buf.packet());
        dev.transmit(tx_buf).unwrap();
        ret
    }
}
