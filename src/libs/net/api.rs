use core::str;
use core::str::FromStr;
use core::mem::ManuallyDrop;
use alloc::boxed::Box;

use smoltcp::time::Duration;

use crate::libs::error::ShyperError;

use super::{Handle, IpAddress};
use super::tcp::AsyncTcpSocket;
use super::tcp::Shutdown;
use super::addr::ipaddr_to_ipaddress;
use super::executor::block_on;

/// Opens a TCP connection to a remote host.
#[inline(always)]
pub fn tcp_stream_connect(
    ip: &[u8],
    port: u16,
    timeout: Option<u64>,
) -> Result<Handle, ShyperError> {
    let local_endpoint = super::interface::get_ephemeral_port()?;
    let socket = Box::new(AsyncTcpSocket::new(local_endpoint));
    let address = IpAddress::from_str(str::from_utf8(ip).map_err(|_| ShyperError::InvalidInput)?)
        .map_err(|_| ShyperError::InvalidInput)?;
    debug!(
        "tcp_stream_connect T[{}] to {}:{}",
        crate::libs::thread::current_thread_id(),
        address,
        port
    );
    block_on(
        socket.connect(address, port, local_endpoint),
        timeout.map(Duration::from_millis),
    )??;
    debug!(
        "tcp_stream_connect T[{}] to {}:{} success local_endpoint {}",
        crate::libs::thread::current_thread_id(),
        address,
        port,
        local_endpoint
    );
    Ok(Handle(Box::into_raw(socket) as *mut _ as usize))
}

#[inline(always)]
pub fn tcp_stream_read(handle: Handle, buffer: &mut [u8]) -> Result<usize, ShyperError> {
    let socket = ManuallyDrop::new(Box::<AsyncTcpSocket>::from(handle));
    // let peer_addr = tcp_stream_peer_addr(handle)?;
    // debug!(
    //     "tcp_stream_read on Thread {} from {}:{}",
    //     crate::libs::thread::current_thread_id(),
    //     peer_addr.0,
    //     peer_addr.1
    // );
    block_on(socket.read(buffer), None)?
}

#[inline(always)]
pub fn tcp_stream_write(handle: Handle, buffer: &[u8]) -> Result<usize, ShyperError> {
    let socket = ManuallyDrop::new(Box::<AsyncTcpSocket>::from(handle));
    // let peer_addr = tcp_stream_peer_addr(handle)?;
    // let s = match str::from_utf8(buffer) {
    //     Ok(v) => v,
    //     Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    // };
    // debug!(
    //     "tcp_stream_write T[{}] to {}:{}, len {},\n{}",
    //     crate::libs::thread::current_thread_id(),
    //     peer_addr.0,
    //     peer_addr.1,
    //     buffer.len(),
    //     // buffer,
    //     s
    // );
    block_on(socket.write(buffer), None)?
}

/// Close a TCP connection
#[inline(always)]
pub fn tcp_stream_close(handle: Handle) -> Result<(), ShyperError> {
    let peer_addr = tcp_stream_peer_addr(handle)?;
    debug!(
        "tcp_stream_close T[{}] ip {}:{}",
        crate::libs::thread::current_thread_id(),
        peer_addr.0,
        peer_addr.1
    );
    let socket = Box::<AsyncTcpSocket>::from(handle);
    block_on(socket.close(), None)?
}

#[inline(always)]
pub fn tcp_stream_shutdown(handle: Handle, how: Shutdown) -> Result<(), ShyperError> {
    match how {
        Shutdown::Read => {
            // warn!("Shutdown::Read is not implemented");
            Ok(())
        }
        Shutdown::Write => tcp_stream_close(handle),
        Shutdown::Both => tcp_stream_close(handle),
    }
}

#[inline(always)]
pub fn tcp_stream_peer_addr(handle: Handle) -> Result<(IpAddress, u16), ShyperError> {
    let socket = ManuallyDrop::new(Box::<AsyncTcpSocket>::from(handle));

    let peer_addr = socket.peer_addr().unwrap();
    let (addr, port) = (ipaddr_to_ipaddress(peer_addr.ip()), peer_addr.port());

    Ok((addr, port))
}

#[inline(always)]
pub fn tcp_stream_socket_addr(handle: Handle) -> Result<(IpAddress, u16), ShyperError> {
    let socket = ManuallyDrop::new(Box::<AsyncTcpSocket>::from(handle));

    let local_addr = socket.local_addr().unwrap();
    let (addr, port) = (ipaddr_to_ipaddress(local_addr.ip()), local_addr.port());

    Ok((addr, port))
}

#[inline(always)]
pub fn tcp_listener_bind(ip: &[u8], port: u16) -> Result<u16, ShyperError> {
    let ip = str::from_utf8(ip).map_err(|_| ShyperError::InvalidInput)?;
    let port = if port == 0 {
        super::interface::get_ephemeral_port()?
    } else if !super::interface::check_local_endpoint(port) {
        port
    } else {
        warn!("tcp_listener_bind failed, port has been occupied");
        return Err(ShyperError::ConnectionRefused);
    };
    debug!(
        "tcp_listener_bind T[{}] success on ip {:?} port {}",
        crate::libs::thread::current_thread_id(),
        ip,
        port
    );
    Ok(port)
}

/// Wait for connection at specified address.
#[inline(always)]
pub fn tcp_listener_accept(port: u16) -> Result<(Handle, IpAddress, u16), ShyperError> {
    let local_endpoint = port;
    let socket = Box::new(AsyncTcpSocket::new(local_endpoint));
    block_on(socket.accept(), None)??;

    debug!(
        "tcp_listener_accept on Thread {} success on ip {:?}, local_endpoint {}",
        crate::libs::thread::current_thread_id(),
        socket.local_addr().unwrap(),
        local_endpoint
    );

    let peer_addr = socket.peer_addr().unwrap();
    let (addr, port) = (ipaddr_to_ipaddress(peer_addr.ip()), peer_addr.port());

    let handle = Handle(Box::into_raw(socket) as *mut _ as usize);

    Ok((handle, addr, port))
}

/// If set, this option disables the Nagle algorithm. This means that segments are
/// always sent as soon as possible, even if there is only a small amount of data.
/// When not set, data is buffered until there is a sufficient amount to send out,
/// thereby avoiding the frequent sending of small packets.
#[inline(always)]
pub fn tcp_stream_set_no_delay(handle: Handle, mode: bool) -> Result<(), ShyperError> {
    let socket = ManuallyDrop::new(Box::<AsyncTcpSocket>::from(handle));
    socket.set_no_delay(mode)
}

#[inline(always)]
pub fn tcp_stream_no_delay(handle: Handle) -> Result<bool, ShyperError> {
    let socket = ManuallyDrop::new(Box::<AsyncTcpSocket>::from(handle));
    socket.no_delay()
}

#[inline(always)]
pub fn tcp_stream_set_nonblocking(handle: Handle, mode: bool) -> Result<(), ShyperError> {
    // non-blocking mode is currently not support
    // => return only an error, if `mode` is defined as `true`
    let socket = ManuallyDrop::new(Box::<AsyncTcpSocket>::from(handle));
    socket.set_nonblocking(mode)
}

#[inline(always)]
pub fn tcp_stream_set_read_timeout(
    _handle: Handle,
    timeout: Option<u64>,
) -> Result<(), ShyperError> {
    if timeout.is_none() {
        return Ok(());
    }
    warn!("tcp_stream_set_read_timeout is not supported");
    Err(ShyperError::Unsupported)
}

#[inline(always)]
pub fn tcp_stream_get_read_timeout(_handle: Handle) -> Result<Option<u64>, ShyperError> {
    warn!("tcp_stream_get_read_timeout is not supported");
    Ok(None)
}

#[inline(always)]
pub fn tcp_stream_set_write_timeout(
    _handle: Handle,
    timeout: Option<u64>,
) -> Result<(), ShyperError> {
    if timeout.is_none() {
        return Ok(());
    }
    warn!("tcp_stream_set_write_timeout is not supported");
    Err(ShyperError::Unsupported)
}

#[inline(always)]
pub fn tcp_stream_get_write_timeout(_handle: Handle) -> Result<Option<u64>, ShyperError> {
    warn!("tcp_stream_get_write_timeout is not supported");
    Ok(None)
}

#[inline(always)]
pub fn tcp_stream_duplicate(_handle: Handle) -> Result<Handle, ShyperError> {
    warn!("tcp_stream_duplicate is not supported");
    Err(ShyperError::Unsupported)
}

#[inline(always)]
pub fn tcp_stream_peek(_handle: Handle, _buf: &mut [u8]) -> Result<usize, ShyperError> {
    warn!("tcp_stream_peek is not supported");
    Err(ShyperError::Unsupported)
}

#[inline(always)]
pub fn tcp_stream_set_tll(_handle: Handle, _ttl: u32) -> Result<(), ShyperError> {
    warn!("tcp_stream_set_tll is not supported");
    Err(ShyperError::Unsupported)
}

#[inline(always)]
pub fn tcp_stream_get_tll(_handle: Handle) -> Result<u32, ShyperError> {
    warn!("tcp_stream_get_tll is not supported");
    Err(ShyperError::Unsupported)
}
