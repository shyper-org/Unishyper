pub mod addr;
/// Network stack implement on smoltcp
#[cfg_attr(feature = "axdriver", path = "axdevice.rs")]
mod device;
mod interface;

// tcp api
pub mod tcp;
// pub(crate) use tcp::*;

// udp api
mod udp;
pub(crate) use udp::AsyncUdpSocket as UdpSocket;

pub mod api;

pub(crate) use interface::network_init as init;
pub(crate) use interface::network_poll;
pub(crate) use interface::now;
pub(crate) use interface::NIC;

pub(crate) type SmoltcpSocketHandle = smoltcp::iface::SocketHandle;

pub type Handle = i32;

// pub(crate) use smoltcp::wire::IpAddress;
