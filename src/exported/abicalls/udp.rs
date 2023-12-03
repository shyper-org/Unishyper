use core::time::Duration;
use core::net::SocketAddr;

use crate::libs::net::api;
use crate::libs::net::Handle;

#[no_mangle]
pub fn shyper_udp_socket() -> Result<Handle, ()> {
    api::udp_socket().map_err(|_| ())
}

#[no_mangle]
pub fn shyper_udp_bind(_fd: Handle, _addr: SocketAddr, _timeout: Option<u64>) -> Result<(), ()> {
    unimplemented!()
}

#[no_mangle]
pub fn shyper_udp_peer_addr(_fd: Handle) -> Result<SocketAddr, ()> {
    unimplemented!()
}

#[no_mangle]
pub fn shyper_udp_socket_addr(_fd: Handle) -> Result<SocketAddr, ()> {
    unimplemented!()
}

#[no_mangle]
pub fn shyper_udp_recv_from(_fd: Handle, _buffer: &mut [u8]) -> Result<(usize, SocketAddr), ()> {
    unimplemented!()
}

#[no_mangle]
pub fn shyper_udp_peek_from(_fd: Handle, _buffer: &mut [u8]) -> Result<(usize, SocketAddr), ()> {
    unimplemented!()
}

#[no_mangle]
pub fn shyper_udp_send_to(_fd: Handle, _buffer: &[u8], _: &SocketAddr) -> Result<usize, ()> {
    unimplemented!()
}

#[no_mangle]
pub fn shyper_udp_duplicate(_fd: Handle) -> Result<Handle, ()> {
    unimplemented!()
}

pub fn shyper_udp_set_read_timeout(_: Option<Duration>) -> Result<(), ()> {
    unimplemented!()
}

pub fn shyper_udp_set_write_timeout(_: Option<Duration>) -> Result<(), ()> {
    unimplemented!()
}

pub fn shyper_udp_read_timeout() -> Result<Option<Duration>, ()> {
    unimplemented!()
}

pub fn shyper_udp_write_timeout() -> Result<Option<Duration>, ()> {
    unimplemented!()
}

pub fn shyper_udp_set_broadcast(_: bool) -> Result<(), ()> {
    unimplemented!()
}

pub fn shyper_udp_broadcast() -> Result<bool, ()> {
    unimplemented!()
}

pub fn shyper_udp_set_ttl(_: u32) -> Result<(), ()> {
    unimplemented!()
}

pub fn shyper_udp_ttl() -> Result<u32, ()> {
    unimplemented!()
}

pub fn shyper_udp_set_nonblocking(_: bool) -> Result<(), ()> {
    unimplemented!()
}

pub fn shyper_udp_recv(_: &mut [u8]) -> Result<usize, ()> {
    unimplemented!()
}

pub fn shyper_udp_peek(_: &mut [u8]) -> Result<usize, ()> {
    unimplemented!()
}

pub fn shyper_udp_send(_: &[u8]) -> Result<usize, ()> {
    unimplemented!()
}

pub fn shyper_udp_connect(_: &SocketAddr) -> Result<(), ()> {
    unimplemented!()
}
