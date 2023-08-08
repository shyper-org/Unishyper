use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::fmt;
use core::time::Duration;
use ioslice::{IoSlice, IoSliceMut};
use no_std_net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

use crate::exported::shyperstd::io;
use crate::libs::net::{
    tcp::*,
    Handle,
    IpAddress::{Ipv4, Ipv6},
};

use crate::libs::net::Shutdown;

#[derive(Debug, Clone)]
pub struct Socket(Handle);

impl Socket {
    fn as_inner(&self) -> &Handle {
        &self.0
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = tcp_stream_close(self.0);
    }
}

pub(crate) fn default_read_exact(this: &mut TcpStream, mut buf: &mut [u8]) -> io::Result {
    while !buf.is_empty() {
        match this.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
                // println!("read {}",n);
            }
            Err(e) => return Err(e),
        }
    }
    if !buf.is_empty() {
        Err("failed to fill whole buffer")
    } else {
        Ok(())
    }
}

// Arc is used to count the number of used sockets.
// Only if all sockets are released, the drop
// method will close the socket.
#[derive(Clone)]
pub struct TcpStream(Arc<Socket>);

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        let addrs = match addr.to_socket_addrs() {
            Ok(addrs) => addrs,
            Err(_) => return Err("ToSocketAddrError"),
        };
        for addr in addrs {
            match tcp_stream_connect(addr.ip().to_string().as_bytes(), addr.port(), None) {
                Ok(handle) => return Ok(TcpStream(Arc::new(Socket(handle)))),
                _ => continue,
            }
        }
        return Err("Unable to initiate a connection on a socket");
    }

    pub fn connect_timeout(saddr: &SocketAddr, duration: Duration) -> io::Result<TcpStream> {
        match tcp_stream_connect(
            saddr.ip().to_string().as_bytes(),
            saddr.port(),
            Some(duration.as_millis() as u64),
        ) {
            Ok(handle) => Ok(TcpStream(Arc::new(Socket(handle)))),
            _ => Err("Unable to initiate a connection on a socket"),
        }
    }

    pub fn set_read_timeout(&self, duration: Option<Duration>) -> io::Result {
        tcp_stream_set_read_timeout(*self.0.as_inner(), duration.map(|d| d.as_millis() as u64))
            .map_err(|_| "Unable to set timeout value")
    }

    pub fn set_write_timeout(&self, duration: Option<Duration>) -> io::Result {
        tcp_stream_set_write_timeout(*self.0.as_inner(), duration.map(|d| d.as_millis() as u64))
            .map_err(|_| ("Unable to set timeout value"))
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        let duration = tcp_stream_get_read_timeout(*self.0.as_inner())
            .map_err(|_| "Unable to determine timeout value")?;

        Ok(duration.map(|d| Duration::from_millis(d)))
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        let duration = tcp_stream_get_write_timeout(*self.0.as_inner())
            .map_err(|_| "Unable to determine timeout value")?;

        Ok(duration.map(|d| Duration::from_millis(d)))
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        tcp_stream_peek(*self.0.as_inner(), buf).map_err(|_| "peek failed")
    }

    pub fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        // self.read_vectored(&mut [IoSliceMut::new(buffer)])
        let ret = tcp_stream_read(*self.0.as_inner(), &mut buffer[0..])
            .map_err(|_| "Unable to read on socket")?;
        Ok(ret)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result {
        default_read_exact(self, buf)
    }

    // Why use vectored???
    pub fn read_vectored(&self, ioslice: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut size: usize = 0;

        for i in ioslice.iter_mut() {
            let ret = tcp_stream_read(*self.0.as_inner(), &mut i[0..])
                .map_err(|_| "Unable to read on socket")?;

            if ret != 0 {
                size += ret;
            }
        }

        Ok(size)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        true
    }

    pub fn write(&self, buffer: &[u8]) -> io::Result<usize> {
        // self.write_vectored(&[IoSlice::new(buffer)])
        let size = tcp_stream_write(*self.0.as_inner(), buffer)
            .map_err(|_| "Unable to write on socket")?;
        Ok(size)
    }

    pub fn write_all(&mut self, mut buf: &[u8]) -> io::Result {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err("failed to write whole buffer"),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    // Why use vectored???
    pub fn write_vectored(&self, ioslice: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut size: usize = 0;

        for i in ioslice.iter() {
            size +=
                tcp_stream_write(*self.0.as_inner(), i).map_err(|_| "Unable to write on socket")?;
        }

        Ok(size)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        true
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        let (ipaddr, port) =
            tcp_stream_peer_addr(*self.0.as_inner()).map_err(|_| ("peer_addr failed"))?;

        let saddr = match ipaddr {
            Ipv4(ref addr) => SocketAddr::new(IpAddr::V4(Ipv4Addr::from(addr.0)), port),
            Ipv6(ref addr) => SocketAddr::new(IpAddr::V6(Ipv6Addr::from(addr.0)), port),
            _ => {
                return Err("peer_addr failed");
            }
        };

        Ok(saddr)
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        unimplemented! {}
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result {
        tcp_stream_shutdown(*self.0.as_inner(), how).map_err(|_| "unable to shutdown socket")
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        Ok(self.clone())
    }

    pub fn set_linger(&self, _linger: Option<Duration>) -> io::Result {
        unimplemented!()
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        unimplemented!()
    }

    pub fn set_nodelay(&self, mode: bool) -> io::Result {
        tcp_stream_set_no_delay(*self.0.as_inner(), mode).map_err(|_| "set_nodelay failed")
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        tcp_stream_no_delay(*self.0.as_inner()).map_err(|_| "nodelay failed")
    }

    pub fn set_ttl(&self, tll: u32) -> io::Result {
        tcp_stream_set_tll(*self.0.as_inner(), tll).map_err(|_| "unable to set TTL")
    }

    pub fn ttl(&self) -> io::Result<u32> {
        tcp_stream_get_tll(*self.0.as_inner()).map_err(|_| "unable to get TTL")
    }

    pub fn take_error(&self) -> io::Result {
        unimplemented!()
    }

    pub fn set_nonblocking(&self, mode: bool) -> io::Result {
        tcp_stream_set_nonblocking(*self.0.as_inner(), mode)
            .map_err(|_| "unable to set blocking mode")
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[derive(Clone)]
pub struct TcpListener(SocketAddr);

impl TcpListener {
    /// Todo: use `bind` provided in `tcplistener`.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        super::each_addr(addr, |addr: io::Result<&SocketAddr>| {
            let addr = addr?;
            Ok(TcpListener(*addr))
        })
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        Ok(self.0)
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (handle, ipaddr, port) =
            tcp_listener_accept(self.0.port()).map_err(|_| "accept failed")?;
        let saddr = match ipaddr {
            Ipv4(ref addr) => SocketAddr::new(IpAddr::V4(Ipv4Addr::from(addr.0)), port),
            Ipv6(ref addr) => SocketAddr::new(IpAddr::V6(Ipv6Addr::from(addr.0)), port),
            _ => {
                return Err("accept failed");
            }
        };

        Ok((TcpStream(Arc::new(Socket(handle))), saddr))
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        Ok(self.clone())
    }

    pub fn set_ttl(&self, _: u32) -> io::Result {
        unimplemented!()
    }

    pub fn ttl(&self) -> io::Result<u32> {
        unimplemented!()
    }

    pub fn set_only_v6(&self, _: bool) -> io::Result {
        unimplemented!()
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        unimplemented!()
    }

    pub fn take_error(&self) -> io::Result {
        unimplemented!()
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result {
        unimplemented!()
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
