/// Shyper unikernel abi for net operations with smoltcp support.
/// See src/libs/net for more details.
use crate::libs::net::api;
use crate::libs::net::Handle;
use crate::libs::net::tcp::Shutdown;

use core::net::SocketAddr;

#[no_mangle]
pub fn shyper_tcp_socket() -> Result<Handle, ()> {
    api::tcp_socket().map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_connect(fd: Handle, addr: SocketAddr, _timeout: Option<u64>) -> Result<(), ()> {
    api::tcp_connect(fd, addr).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_read(fd: Handle, buffer: &mut [u8]) -> Result<usize, ()> {
    api::tcp_read(fd, buffer).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_write(fd: Handle, buffer: &[u8]) -> Result<usize, ()> {
    api::tcp_write(fd, buffer).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_close(fd: Handle) -> Result<(), ()> {
    debug!("shyper_tcp_close fd {}", fd);
    api::tcp_close(fd).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_shutdown(fd: Handle, how: i32) -> Result<(), ()> {
    api::tcp_shutdown(fd, Shutdown::from_i32(how)).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_peer_addr(fd: Handle) -> Result<SocketAddr, ()> {
    api::tcp_peer_addr(fd).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_socket_addr(fd: Handle) -> Result<SocketAddr, ()> {
    api::tcp_socket_addr(fd).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_bind(fd: Handle, addr: SocketAddr) -> Result<(), ()> {
    api::tcp_bind(fd, addr).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_listen(fd: Handle, _backlog: usize) -> Result<(), ()> {
    api::tcp_listen(fd, _backlog).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_accept(fd: Handle) -> Result<(Handle, SocketAddr), ()> {
    api::tcp_accept(fd).map_err(|_| ())
}

/// If set, this option disables the Nagle algorithm. This means that segments are
/// always sent as soon as possible, even if there is only a small amount of data.
/// When not set, data is buffered until there is a sufficient amount to send out,
/// thereby avoiding the frequent sending of small packets.
#[no_mangle]
pub fn shyper_tcp_set_no_delay(fd: Handle, mode: bool) -> Result<(), ()> {
    api::tcp_set_no_delay(fd, mode).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_set_nonblocking(fd: Handle, mode: bool) -> Result<(), ()> {
    api::tcp_set_nonblocking(fd, mode).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_set_read_timeout(fd: Handle, timeout: Option<u64>) -> Result<(), ()> {
    api::tcp_set_read_timeout(fd, timeout).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_get_read_timeout(fd: Handle) -> Result<Option<u64>, ()> {
    api::tcp_get_read_timeout(fd).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_set_write_timeout(fd: Handle, timeout: Option<u64>) -> Result<(), ()> {
    api::tcp_set_write_timeout(fd, timeout).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_get_write_timeout(fd: Handle) -> Result<Option<u64>, ()> {
    api::tcp_get_write_timeout(fd).map_err(|_| ())
}

#[deprecated(since = "0.1.14", note = "Please don't use this function")]
#[no_mangle]
pub fn shyper_tcp_duplicate(fd: Handle) -> Result<Handle, ()> {
    api::tcp_duplicate(fd).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_peek(fd: Handle, buf: &mut [u8]) -> Result<usize, ()> {
    api::tcp_peek(fd, buf).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_set_tll(fd: Handle, ttl: u32) -> Result<(), ()> {
    api::tcp_set_tll(fd, ttl).map_err(|_| ())
}

#[no_mangle]
pub fn shyper_tcp_get_tll(fd: Handle) -> Result<u32, ()> {
    api::tcp_get_tll(fd).map_err(|_| ())
}
