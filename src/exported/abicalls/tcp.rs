/// Shyper unikernel abi for net operations with smoltcp support.
/// See src/libs/net for more details.
use crate::libs::net::api;
use crate::libs::net::Handle;
use crate::libs::net::tcp::Shutdown;

use smoltcp::wire::IpAddress;

#[no_mangle]
pub fn shyper_tcp_stream_connect(ip: &[u8], port: u16, timeout: Option<u64>) -> Result<Handle, ()> {
    api::tcp_stream_connect(ip, port, timeout).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_read(handle: Handle, buffer: &mut [u8]) -> Result<usize, ()> {
    api::tcp_stream_read(handle, buffer).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_write(handle: Handle, buffer: &[u8]) -> Result<usize, ()> {
    api::tcp_stream_write(handle, buffer).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_close(handle: Handle) -> Result<(), ()> {
    api::tcp_stream_close(handle).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_shutdown(handle: Handle, how: i32) -> Result<(), ()> {
    api::tcp_stream_shutdown(handle, Shutdown::from_i32(how)).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_peer_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
    api::tcp_stream_peer_addr(handle).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_socket_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
    api::tcp_stream_socket_addr(handle).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_listener_bind(ip: &[u8], port: u16) -> Result<u16, ()> {
    api::tcp_listener_bind(ip, port).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_listener_accept(port: u16) -> Result<(Handle, IpAddress, u16), ()> {
    let res = api::tcp_listener_accept(port).map_err(|_| ());
    debug!("shyper_tcp_listener_accept : {port} res {:?}", res);
    res
}

/// If set, this option disables the Nagle algorithm. This means that segments are
/// always sent as soon as possible, even if there is only a small amount of data.
/// When not set, data is buffered until there is a sufficient amount to send out,
/// thereby avoiding the frequent sending of small packets.
#[no_mangle]
pub fn shyper_tcp_stream_set_no_delay(handle: Handle, mode: bool) -> Result<(), ()> {
    api::tcp_stream_set_no_delay(handle, mode).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_set_nonblocking(handle: Handle, mode: bool) -> Result<(), ()> {
    api::tcp_stream_set_nonblocking(handle, mode).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_set_read_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
    api::tcp_stream_set_read_timeout(handle, timeout).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_get_read_timeout(handle: Handle) -> Result<Option<u64>, ()> {
    api::tcp_stream_get_read_timeout(handle).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_set_write_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
    api::tcp_stream_set_write_timeout(handle, timeout).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_get_write_timeout(handle: Handle) -> Result<Option<u64>, ()> {
    api::tcp_stream_get_write_timeout(handle).map_err(|_| ())
}

#[deprecated(since = "0.1.14", note = "Please don't use this function")]
#[no_mangle]
pub fn shyper_tcp_stream_duplicate(handle: Handle) -> Result<Handle, ()> {
    api::tcp_stream_duplicate(handle).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_peek(handle: Handle, buf: &mut [u8]) -> Result<usize, ()> {
    api::tcp_stream_peek(handle, buf).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_set_tll(handle: Handle, ttl: u32) -> Result<(), ()> {
    api::tcp_stream_set_tll(handle, ttl).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_stream_get_tll(handle: Handle) -> Result<u32, ()> {
    api::tcp_stream_get_tll(handle).map_err(|_| ())
}
