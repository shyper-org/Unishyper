// tcp api
pub mod tcplistener;
pub mod tcpstream;
// network implement on smoltcp
pub mod interface;
pub mod device;
mod executor;
mod waker;

pub use smoltcp::wire::IpAddress;
pub use smoltcp::socket::SocketHandle as Handle;

/// A handle, identifying a socket
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
// pub struct Handle(usize);

/// initialize the network stack
pub fn network_init() -> i32 {
	info!("network init\n");
    0
}

// /// Internet protocol version.
// #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// pub enum Version {
// 	Unspecified,
// 	Ipv4,
// 	Ipv6,
// }

// /// A four-octet IPv4 address.
// #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
// pub struct Ipv4Address(pub [u8; 4]);

// /// A sixteen-octet IPv6 address.
// #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
// pub struct Ipv6Address(pub [u8; 16]);

// /// An internetworking address.
// #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// pub enum IpAddress {
// 	/// An unspecified address.
// 	/// May be used as a placeholder for storage where the address is not assigned yet.
// 	Unspecified,
// 	/// An IPv4 address.
// 	Ipv4(Ipv4Address),
// 	/// An IPv6 address.
// 	Ipv6(Ipv6Address),
// }
