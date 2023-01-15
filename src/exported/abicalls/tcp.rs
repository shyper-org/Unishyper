/// Shyper unikernel abi for net operations with smoltcp support.
/// See src/libs/net for more details.
use crate::libs::net;
use crate::libs::net::Handle;

use smoltcp::wire::IpAddress;

#[no_mangle]
pub fn shyper_tcp_stream_connect(ip: &[u8], port: u16, timeout: Option<u64>) -> Result<Handle, ()> {
    net::tcp_stream_connect(ip, port, timeout)
}

#[no_mangle]
pub fn shyper_tcp_stream_read(handle: Handle, buffer: &mut [u8]) -> Result<usize, ()> {
    net::tcp_stream_read(handle, buffer)
}

#[no_mangle]
pub fn shyper_tcp_stream_write(handle: Handle, buffer: &[u8]) -> Result<usize, ()> {
    net::tcp_stream_write(handle, buffer)
}

#[no_mangle]
pub fn shyper_tcp_stream_close(handle: Handle) -> Result<(), ()> {
    net::tcp_stream_close(handle)
}

//ToDo: an enum, or at least constants would be better
#[no_mangle]
pub fn shyper_tcp_stream_shutdown(handle: Handle, how: i32) -> Result<(), ()> {
    net::tcp_stream_shutdown(handle, how)
}

#[no_mangle]
pub fn shyper_tcp_stream_peer_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
    net::tcp_stream_peer_addr(handle)
}

#[no_mangle]
pub fn shyper_tcp_stream_socket_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
    net::tcp_stream_socket_addr(handle)
}

#[no_mangle]
pub fn shyper_tcp_listener_bind(ip: &[u8], port: u16) -> Result<u16, ()> {
    net::tcp_listener_bind(ip, port)
}

#[no_mangle]
pub fn shyper_tcp_listener_accept(port: u16) -> Result<(Handle, IpAddress, u16), ()> {
    net::tcp_listener_accept(port)
}

/// If set, this option disables the Nagle algorithm. This means that segments are
/// always sent as soon as possible, even if there is only a small amount of data.
/// When not set, data is buffered until there is a sufficient amount to send out,
/// thereby avoiding the frequent sending of small packets.
#[no_mangle]
pub fn shyper_tcp_set_no_delay(handle: Handle, mode: bool) -> Result<(), ()> {
    net::tcp_set_no_delay(handle, mode)
}

#[no_mangle]
pub fn shyper_tcp_stream_set_nonblocking(handle: Handle, mode: bool) -> Result<(), ()> {
    net::tcp_stream_set_nonblocking(handle, mode)
}

#[no_mangle]
pub fn shyper_tcp_stream_set_read_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
    net::tcp_stream_set_read_timeout(handle, timeout)
}

#[no_mangle]
pub fn shyper_tcp_stream_get_read_timeout(handle: Handle) -> Result<Option<u64>, ()> {
    net::tcp_stream_get_read_timeout(handle)
}

#[no_mangle]
pub fn shyper_tcp_stream_set_write_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
    net::tcp_stream_set_write_timeout(handle, timeout)
}

#[no_mangle]
pub fn shyper_tcp_stream_get_write_timeout(handle: Handle) -> Result<Option<u64>, ()> {
    net::tcp_stream_get_write_timeout(handle)
}

#[deprecated(since = "0.1.14", note = "Please don't use this function")]
#[no_mangle]
pub fn shyper_tcp_stream_duplicate(handle: Handle) -> Result<Handle, ()> {
    net::tcp_stream_duplicate(handle)
}

#[no_mangle]
pub fn shyper_tcp_stream_peek(handle: Handle, buf: &mut [u8]) -> Result<usize, ()> {
    net::tcp_stream_peek(handle, buf)
}

#[no_mangle]
pub fn shyper_tcp_stream_set_tll(handle: Handle, ttl: u32) -> Result<(), ()> {
    net::tcp_stream_set_tll(handle, ttl)
}

#[no_mangle]
pub fn shyper_tcp_stream_get_tll(handle: Handle) -> Result<u32, ()> {
    net::tcp_stream_get_tll(handle)
}
