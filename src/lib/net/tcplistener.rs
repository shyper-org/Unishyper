use super::*;
use super::interface::*;

/// Wait for connection at specified address.
#[inline(always)]
pub fn accept(port: u16) -> Result<(Handle, IpAddress, u16), ()> {
	tcp_listener_accept(port)
}