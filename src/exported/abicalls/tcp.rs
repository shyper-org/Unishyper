use crate::libs::net::Handle;

use smoltcp::wire::IpAddress;

#[no_mangle]
pub fn shyper_tcp_stream_connect(_ip: &[u8], _port: u16, _timeout: Option<u64>) -> Result<Handle, ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_read(_handle: Handle, _buffer: &mut [u8]) -> Result<usize, ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_write(_handle: Handle, _buffer: &[u8]) -> Result<usize, ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_close(_handle: Handle) -> Result<(), ()> {
	Err(())
}

//ToDo: an enum, or at least constants would be better
#[no_mangle]
pub fn shyper_tcp_stream_shutdown(_handle: Handle, _how: i32) -> Result<(), ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_set_read_timeout(_handle: Handle, _timeout: Option<u64>) -> Result<(), ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_get_read_timeout(_handle: Handle) -> Result<Option<u64>, ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_set_write_timeout(_handle: Handle, _timeout: Option<u64>) -> Result<(), ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_get_write_timeout(_handle: Handle) -> Result<Option<u64>, ()> {
	Err(())
}

#[deprecated(since = "0.1.14", note = "Please don't use this function")]
#[no_mangle]
pub fn shyper_tcp_stream_duplicate(_handle: Handle) -> Result<Handle, ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_peek(_handle: Handle, _buf: &mut [u8]) -> Result<usize, ()> {
	Err(())
}

/// If set, this option disables the Nagle algorithm. This means that segments are
/// always sent as soon as possible, even if there is only a small amount of data.
/// When not set, data is buffered until there is a sufficient amount to send out,
/// thereby avoiding the frequent sending of small packets.
#[no_mangle]
pub fn shyper_tcp_set_no_delay(_handle: Handle, _mode: bool) -> Result<(), ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_set_nonblocking(_handle: Handle, _mode: bool) -> Result<(), ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_set_tll(_handle: Handle, _ttl: u32) -> Result<(), ()> {
	Err(())
}

#[no_mangle]
pub fn shyper_tcp_stream_get_tll(_handle: Handle) -> Result<u32, ()> {
	Err(())
}

#[cfg(feature = "tcp")]
#[no_mangle]
pub fn shyper_tcp_stream_peer_addr(_handle: Handle) -> Result<(IpAddress, u16), ()> {
	Err(())
}

#[cfg(feature = "tcp")]
#[no_mangle]
pub fn shyper_tcp_listener_accept(_port: u16) -> Result<(Handle, IpAddress, u16), ()> {
	Err(())
}
