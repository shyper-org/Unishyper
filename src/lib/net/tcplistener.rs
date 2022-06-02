use super::{Handle, IpAddress};

extern "Rust" {
	fn tcp_listener_accept(port: u16) -> Result<(Handle, IpAddress, u16), ()>;
}

/// Wait for connection at specified address.
#[inline(always)]
pub fn accept(port: u16) -> Result<(Handle, IpAddress, u16), ()> {
	unsafe { tcp_listener_accept(port) }
}