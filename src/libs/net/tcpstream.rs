//! `tcpstream` provide an interface to establish tcp socket client.
use super::*;
use super::interface::*;

/// Opens a TCP connection to a remote host.
#[inline(always)]
pub fn connect(ip: &[u8], port: u16, timeout: Option<u64>) -> Result<Handle, ()> {
    tcp_stream_connect(ip, port, timeout)
}

/// Close a TCP connection
#[inline(always)]
pub fn close(handle: Handle) -> Result<(), ()> {
    tcp_stream_close(handle)
}

#[inline(always)]
pub fn peek(handle: Handle, buf: &mut [u8]) -> Result<usize, ()> {
    tcp_stream_peek(handle, buf)
}

#[inline(always)]
pub fn peer_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
    tcp_stream_peer_addr(handle)
}
#[inline(always)]
pub fn read(handle: Handle, buffer: &mut [u8]) -> Result<usize, ()> {
    tcp_stream_read(handle, buffer)
}

#[inline(always)]
pub fn write(handle: Handle, buffer: &[u8]) -> Result<usize, ()> {
    tcp_stream_write(handle, buffer)
}

#[inline(always)]
pub fn set_read_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
    tcp_stream_set_read_timeout(handle, timeout)
}

#[inline(always)]
pub fn set_write_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
    tcp_stream_set_write_timeout(handle, timeout)
}

#[inline(always)]
pub fn get_read_timeout(handle: Handle) -> Result<Option<u64>, ()> {
    tcp_stream_get_read_timeout(handle)
}

#[inline(always)]
pub fn get_write_timeout(handle: Handle) -> Result<Option<u64>, ()> {
    tcp_stream_get_write_timeout(handle)
}

#[inline(always)]
pub fn set_nodelay(_: Handle, mode: bool) -> Result<(), ()> {
    // smoltcp does not support Nagle's algorithm
    // => to enable Nagle's algorithm isn't possible
    if mode {
        Ok(())
    } else {
        Err(())
    }
}

#[inline(always)]
pub fn nodelay(_: Handle) -> Result<bool, ()> {
    // smoltcp does not support Nagle's algorithm
    // => return always true
    Ok(true)
}

#[inline(always)]
pub fn set_nonblocking(handle: Handle, mode: bool) -> Result<(), ()> {
    tcp_stream_set_nonblocking(handle, mode)
}

#[inline(always)]
pub fn set_tll(handle: Handle, ttl: u32) -> Result<(), ()> {
    tcp_stream_set_tll(handle, ttl)
}

#[inline(always)]
pub fn get_tll(handle: Handle) -> Result<u32, ()> {
    tcp_stream_get_tll(handle)
}

#[inline(always)]
pub fn shutdown(handle: Handle, how: Shutdown) -> Result<(), ()> {
    tcp_stream_shutdown(handle, how)
}
