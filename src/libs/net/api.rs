use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicI32, Ordering};

use pflock::PFLock;

use crate::libs::error::ShyperError;

use super::Handle;
use super::tcp::TcpSocket;
use super::tcp::Shutdown;
use super::addr::SocketAddr;

/// Atomic counter to determine the next unused file descriptor.
static FD_COUNTER: AtomicI32 = AtomicI32::new(3);

/// Mapping between file descriptor and the referenced object.
static FD_MAP: PFLock<BTreeMap<Handle, Arc<TcpSocket>>> =
    PFLock::new(BTreeMap::<Handle, Arc<TcpSocket>>::new());

pub(crate) fn get_object(fd: Handle) -> Result<Arc<TcpSocket>, ShyperError> {
    Ok((*(FD_MAP.read().get(&fd).ok_or(ShyperError::NotFound)?)).clone())
}

pub(crate) fn remove_object(fd: Handle) -> Result<Arc<TcpSocket>, ShyperError> {
    if fd <= 2 {
        Err(ShyperError::InvalidInput)
    } else {
        Ok(FD_MAP.write().remove(&fd).ok_or(ShyperError::NotFound)?)
    }
}

pub(crate) fn insert_object(fd: Handle, obj: TcpSocket) -> Option<Arc<TcpSocket>> {
    FD_MAP.write().insert(fd, Arc::new(obj))
}

#[inline(always)]
pub fn tcp_socket() -> Result<Handle, ShyperError> {
    let fd = FD_COUNTER.fetch_add(1, Ordering::SeqCst);
    let socket = TcpSocket::new();
    let _ = insert_object(fd, socket);
    Ok(fd)
}

/// Opens a TCP connection to a remote host.
#[inline(always)]
pub fn tcp_stream_connect(fd: Handle, addr: SocketAddr) -> Result<(), ShyperError> {
    get_object(fd)?.connect(addr)
}

#[inline(always)]
pub fn tcp_stream_read(fd: Handle, buffer: &mut [u8]) -> Result<usize, ShyperError> {
    get_object(fd)?.read(buffer)
}

#[inline(always)]
pub fn tcp_stream_write(fd: Handle, buffer: &[u8]) -> Result<usize, ShyperError> {
    get_object(fd)?.write(buffer)
}

/// Close a TCP connection
#[inline(always)]
pub fn tcp_close(fd: Handle) -> Result<(), ShyperError> {
    let socket = remove_object(fd)?;
    // See `Drop` implemented for `TcpSocket`.
    drop(socket);

    Ok(())
}

#[inline(always)]
pub fn tcp_stream_shutdown(_fd: Handle, _how: Shutdown) -> Result<(), ShyperError> {
    // match how {
    //     Shutdown::Read => {
    //         // warn!("Shutdown::Read is not implemented");
    //         Ok(())
    //     }
    //     Shutdown::Write => tcp_stream_close(handle),
    //     Shutdown::Both => tcp_stream_close(handle),
    // }
    Ok(())
}

#[inline(always)]
pub fn tcp_stream_peer_addr(fd: Handle) -> Result<SocketAddr, ShyperError> {
    get_object(fd)?.peer_addr()
}

#[inline(always)]
pub fn tcp_socket_addr(fd: Handle) -> Result<SocketAddr, ShyperError> {
    get_object(fd)?.local_addr()
}

#[inline(always)]
pub fn tcp_bind(fd: Handle, addr: SocketAddr) -> Result<(), ShyperError> {
    let socket = get_object(fd)?;
    socket.bind(addr)
}

#[inline(always)]
pub fn tcp_listen(fd: Handle, _backlog: usize) -> Result<(), ShyperError> {
    let socket = get_object(fd)?;
    socket.listen()
}

/// Wait for connection at specified address.
#[inline(always)]
pub fn tcp_accept(fd: Handle) -> Result<(Handle, SocketAddr), ShyperError> {
    let socket = get_object(fd)?;
    let new_socket = socket.accept()?;

    let peer_addr = new_socket.peer_addr()?;

    debug!(
        "tcp_listener_accept on Thread {} success on {}, remote {}",
        crate::libs::thread::current_thread_id(),
        new_socket.local_addr()?,
        new_socket.peer_addr()?,
    );

    let new_fd = FD_COUNTER.fetch_add(1, Ordering::SeqCst);

    let _ = insert_object(new_fd, new_socket);

    Ok((new_fd, peer_addr))
}

/// If set, this option disables the Nagle algorithm. This means that segments are
/// always sent as soon as possible, even if there is only a small amount of data.
/// When not set, data is buffered until there is a sufficient amount to send out,
/// thereby avoiding the frequent sending of small packets.
#[inline(always)]
pub fn tcp_stream_set_no_delay(fd: Handle, mode: bool) -> Result<(), ShyperError> {
    get_object(fd)?.set_no_delay(mode)
}

#[inline(always)]
pub fn tcp_stream_no_delay(fd: Handle) -> Result<bool, ShyperError> {
    get_object(fd)?.no_delay()
}

#[inline(always)]
pub fn tcp_stream_set_nonblocking(fd: Handle, mode: bool) -> Result<(), ShyperError> {
    // non-blocking mode is currently not support
    // => return only an error, if `mode` is defined as `true`

    get_object(fd)?.set_nonblocking(mode)
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
