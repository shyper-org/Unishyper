use alloc::fmt;
use core::time::Duration;
use ioslice::{IoSlice, IoSliceMut};
use super::{SocketAddr, ToSocketAddrs};

use crate::exported::shyperstd::io;
use crate::libs::error::ShyperError;
use crate::libs::net::{api, Handle};

pub use crate::libs::net::tcp::Shutdown;

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
pub struct TcpStream(Handle);

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        super::each_addr(addr, |addr: io::Result<&SocketAddr>| {
            let addr = *addr?;
            let fd = api::tcp_socket()?;
            api::tcp_stream_connect(fd, addr)?;

            Ok(TcpStream(fd))
        })
    }

    pub fn set_read_timeout(&self, duration: Option<Duration>) -> io::Result {
        api::tcp_stream_set_read_timeout(self.0, duration.map(|d| d.as_millis() as u64))
    }

    pub fn set_write_timeout(&self, duration: Option<Duration>) -> io::Result {
        api::tcp_stream_set_write_timeout(self.0, duration.map(|d| d.as_millis() as u64))
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        let duration = api::tcp_stream_get_read_timeout(self.0)?;

        Ok(duration.map(|d| Duration::from_millis(d)))
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        let duration = api::tcp_stream_get_write_timeout(self.0)?;

        Ok(duration.map(|d| Duration::from_millis(d)))
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        api::tcp_stream_peek(self.0, buf)
    }

    pub fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        // self.read_vectored(&mut [IoSliceMut::new(buffer)])
        let ret = api::tcp_stream_read(self.0, &mut buffer[0..])?;
        Ok(ret)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result {
        default_read_exact(self, buf)
    }

    // Why use vectored???
    pub fn read_vectored(&self, ioslice: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut size: usize = 0;

        for i in ioslice.iter_mut() {
            let ret = api::tcp_stream_read(self.0, &mut i[0..])?;

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
        let size = api::tcp_stream_write(self.0, buffer)?;
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
            size += api::tcp_stream_write(self.0, i)?;
        }

        Ok(size)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        true
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        api::tcp_stream_peer_addr(self.0)
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        unimplemented! {}
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result {
        api::tcp_stream_shutdown(self.0, how)
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
        api::tcp_stream_set_no_delay(self.0, mode)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        api::tcp_stream_no_delay(self.0)
    }

    pub fn set_ttl(&self, tll: u32) -> io::Result {
        api::tcp_stream_set_tll(self.0, tll)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        api::tcp_stream_get_tll(self.0)
    }

    pub fn take_error(&self) -> io::Result {
        unimplemented!()
    }

    pub fn set_nonblocking(&self, mode: bool) -> io::Result {
        api::tcp_stream_set_nonblocking(self.0, mode)
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        debug!("Dropping TcpStream of fd {}", self.0);
        api::tcp_close(self.0).expect("TcpStream close failed");
    }
}

#[derive(Clone)]
pub struct TcpListener(Handle);

impl TcpListener {
    /// Todo: use `bind` provided in `tcplistener`.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        super::each_addr(addr, |addr: io::Result<&SocketAddr>| {
            let fd = api::tcp_socket()?;
            let addr = *addr?;
            let backlog = 128;
            api::tcp_bind(fd, addr)?;
            api::tcp_listen(fd, backlog)?;
            Ok(TcpListener(fd))
        })
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        api::tcp_socket_addr(self.0)
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (fd, socket_addr) = api::tcp_accept(self.0)?;

        Ok((TcpStream(fd), socket_addr))
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        unimplemented!()

        // Ok(self.clone())
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

impl Drop for TcpListener {
    fn drop(&mut self) {
        debug!("Dropping TcpListener!");
        api::tcp_close(self.0).expect("TcpListener close failed");
    }
}
