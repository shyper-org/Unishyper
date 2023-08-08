/// Network stack implement on smoltcp
mod device;
mod executor;
mod addr;
mod interface;

// tcp api
pub mod tcp;
pub(crate) use tcp::*;

// udp api
mod udp;
pub(crate) use udp::AsyncUdpSocket as UdpSocket;

pub(crate) use interface::network_init;
pub(crate) use interface::network_poll;
pub(crate) use interface::now;
pub(crate) use interface::NIC;

pub(crate) type Handle = smoltcp::iface::SocketHandle;

pub(crate) use smoltcp::wire::IpAddress;

