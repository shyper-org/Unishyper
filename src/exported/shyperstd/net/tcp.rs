use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::fmt;
use core::time::Duration;
use ioslice::{IoSlice, IoSliceMut};
use no_std_net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

use crate::exported::shyperstd::io;
use crate::libs::error::ShyperError;
use crate::libs::net::{api, Handle, IpAddress::Ipv4, IpAddress::Ipv6};

use crate::libs::net::tcp::Shutdown;

#[derive(Debug, Clone)]
pub struct Socket(Handle);

impl Socket {
    fn as_inner(&self) -> &Handle {
        &self.0
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = api::tcp_stream_close(self.0);
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
        Err(ShyperError::Io)
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
        super::each_addr(addr, |addr: io::Result<&SocketAddr>| {
            let addr = addr?;
            let handle =
                api::tcp_stream_connect(addr.ip().to_string().as_bytes(), addr.port(), None)?;
            Ok(TcpStream(Arc::new(Socket(handle))))
        })
    }

    pub fn set_read_timeout(&self, duration: Option<Duration>) -> io::Result {
        api::tcp_stream_set_read_timeout(*self.0.as_inner(), duration.map(|d| d.as_millis() as u64))
    }

    pub fn set_write_timeout(&self, duration: Option<Duration>) -> io::Result {
        api::tcp_stream_set_write_timeout(
            *self.0.as_inner(),
            duration.map(|d| d.as_millis() as u64),
        )
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        let duration = api::tcp_stream_get_read_timeout(*self.0.as_inner())?;

        Ok(duration.map(|d| Duration::from_millis(d)))
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        let duration = api::tcp_stream_get_write_timeout(*self.0.as_inner())?;

        Ok(duration.map(|d| Duration::from_millis(d)))
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        api::tcp_stream_peek(*self.0.as_inner(), buf)
    }

    pub fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        // self.read_vectored(&mut [IoSliceMut::new(buffer)])
        let ret = api::tcp_stream_read(*self.0.as_inner(), &mut buffer[0..])?;
        Ok(ret)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result {
        default_read_exact(self, buf)
    }

    // Why use vectored???
    pub fn read_vectored(&self, ioslice: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut size: usize = 0;

        for i in ioslice.iter_mut() {
            let ret = api::tcp_stream_read(*self.0.as_inner(), &mut i[0..])?;

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
        let size = api::tcp_stream_write(*self.0.as_inner(), buffer)?;
        Ok(size)
    }

    pub fn write_all(&mut self, mut buf: &[u8]) -> io::Result {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(ShyperError::UnexpectedEof),
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
            size += api::tcp_stream_write(*self.0.as_inner(), i)?;
        }

        Ok(size)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        true
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        let (ipaddr, port) = api::tcp_stream_peer_addr(*self.0.as_inner())?;

        let saddr = match ipaddr {
            Ipv4(ref addr) => SocketAddr::new(IpAddr::V4(Ipv4Addr::from(addr.0)), port),
            Ipv6(ref addr) => SocketAddr::new(IpAddr::V6(Ipv6Addr::from(addr.0)), port),
        };

        Ok(saddr)
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        unimplemented! {}
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result {
        api::tcp_stream_shutdown(*self.0.as_inner(), how)
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
        api::tcp_stream_set_no_delay(*self.0.as_inner(), mode)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        api::tcp_stream_no_delay(*self.0.as_inner())
    }

    pub fn set_ttl(&self, tll: u32) -> io::Result {
        api::tcp_stream_set_tll(*self.0.as_inner(), tll)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        api::tcp_stream_get_tll(*self.0.as_inner())
    }

    pub fn take_error(&self) -> io::Result {
        unimplemented!()
    }

    pub fn set_nonblocking(&self, mode: bool) -> io::Result {
        api::tcp_stream_set_nonblocking(*self.0.as_inner(), mode)
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
            let mut addr = *addr?;
            let new_port = api::tcp_listener_bind(addr.ip().to_string().as_bytes(), addr.port())?;
            addr.set_port(new_port);
            Ok(TcpListener(addr))
        })
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        Ok(self.0)
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (handle, ipaddr, port) = api::tcp_listener_accept(self.0.port())?;
        let saddr = match ipaddr {
            Ipv4(ref addr) => SocketAddr::new(IpAddr::V4(Ipv4Addr::from(addr.0)), port),
            Ipv6(ref addr) => SocketAddr::new(IpAddr::V6(Ipv6Addr::from(addr.0)), port),
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
