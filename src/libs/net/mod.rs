/// Network stack implement on smoltcp
#[cfg_attr(feature = "axdriver", path = "axdevice.rs")]
mod device;
mod executor;
mod addr;
mod interface;

// tcp api
pub mod tcp;
// pub(crate) use tcp::*;

// udp api
mod udp;
pub(crate) use udp::AsyncUdpSocket as UdpSocket;

pub mod api;

/// Default keep alive interval in milliseconds
const DEFAULT_KEEP_ALIVE_INTERVAL: u64 = 75000;

pub(crate) use interface::network_init as init;
pub(crate) use interface::network_poll;
pub(crate) use interface::now;
pub(crate) use interface::NIC;

pub(crate) type SmoltcpSocketHandle = smoltcp::iface::SocketHandle;

#[derive(Debug, Clone, Copy)]
pub struct Handle(pub usize);


pub(crate) use smoltcp::wire::IpAddress;

