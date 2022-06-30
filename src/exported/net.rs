use core::convert::TryFrom;
use core::time::Duration;
use alloc::{fmt, str};
use alloc::string::ToString;
use no_std_net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use alloc::sync::Arc;
use ioslice::{IoSlice, IoSliceMut};

pub type IoResult<T> = core::result::Result<T, &'static str>;

use crate::lib::net::{
    self,
    Handle,
    IpAddress::{Ipv4, Ipv6},
    tcpstream,
    tcplistener,
};

fn unsupported() -> ! {
    panic!("unsupported function!!\n")
}

pub extern "C" fn network_init() {
    net::init();
}

#[derive(Debug, Clone)]
pub struct Socket(Handle);

impl Socket {
    fn as_inner(&self) -> &Handle {
        &self.0
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = tcpstream::close(self.0);
    }
}

// Arc is used to count the number of used sockets.
// Only if all sockets are released, the drop
// method will close the socket.
#[derive(Clone)]
pub struct TcpStream(Arc<Socket>);

impl TcpStream {
    pub fn connect(addr: IoResult<&SocketAddr>) -> IoResult<TcpStream> {
        let addr = addr?;

        match tcpstream::connect(addr.ip().to_string().as_bytes(), addr.port(), None) {
            Ok(handle) => Ok(TcpStream(Arc::new(Socket(handle)))),
            _ => Err("Unable to initiate a connection on a socket"),
        }
    }

    pub fn connect_timeout(saddr: &SocketAddr, duration: Duration) -> IoResult<TcpStream> {
        match tcpstream::connect(
            saddr.ip().to_string().as_bytes(),
            saddr.port(),
            Some(duration.as_millis() as u64),
        ) {
            Ok(handle) => Ok(TcpStream(Arc::new(Socket(handle)))),
            _ => Err("Unable to initiate a connection on a socket"),
        }
    }

    pub fn set_read_timeout(&self, duration: Option<Duration>) -> IoResult<()> {
        tcpstream::set_read_timeout(*self.0.as_inner(), duration.map(|d| d.as_millis() as u64))
            .map_err(|_| {
                 "Unable to set timeout value"
            })
    }

    pub fn set_write_timeout(&self, duration: Option<Duration>) -> IoResult<()> {
        tcpstream::set_write_timeout(
            *self.0.as_inner(),
            duration.map(|d| d.as_millis() as u64),
        )
        .map_err(|_|  ("Unable to set timeout value"))
    }

    pub fn read_timeout(&self) -> IoResult<Option<Duration>> {
        let duration = tcpstream::get_read_timeout(*self.0.as_inner()).map_err(|_| {
             "Unable to determine timeout value"
        })?;

        Ok(duration.map(|d| Duration::from_millis(d)))
    }

    pub fn write_timeout(&self) -> IoResult<Option<Duration>> {
        let duration = tcpstream::get_write_timeout(*self.0.as_inner()).map_err(|_| {
             "Unable to determine timeout value"
            })?;

        Ok(duration.map(|d| Duration::from_millis(d)))
    }

    pub fn peek(&self, buf: &mut [u8]) -> IoResult<usize> {
        tcpstream::peek(*self.0.as_inner(), buf)
            .map_err(|_|  "peek failed")
    }

    pub fn read(&self, buffer: &mut [u8]) -> IoResult<usize> {
        self.read_vectored(&mut [IoSliceMut::new(buffer)])
    }

    pub fn read_vectored(&self, ioslice: &mut [IoSliceMut<'_>]) -> IoResult<usize> {
        let mut size: usize = 0;

        for i in ioslice.iter_mut() {
            let ret = tcpstream::read(*self.0.as_inner(), &mut i[0..]).map_err(|_| {
                 "Unable to read on socket" })?;

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

    pub fn write(&self, buffer: &[u8]) -> IoResult<usize> {
        self.write_vectored(&[IoSlice::new(buffer)])
    }

    pub fn write_vectored(&self, ioslice: &[IoSlice<'_>]) -> IoResult<usize> {
        let mut size: usize = 0;

        for i in ioslice.iter() {
            size += tcpstream::write(*self.0.as_inner(), i).map_err(|_| {
                "Unable to write on socket"
            })?;
        }

        Ok(size)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        true
    }

    pub fn peer_addr(&self) -> IoResult<SocketAddr> {
        let (ipaddr, port) = tcpstream::peer_addr(*self.0.as_inner())
            .map_err(|_|  ("peer_addr failed"))?;

        let saddr = match ipaddr {
            Ipv4(ref addr) => SocketAddr::new(IpAddr::V4(Ipv4Addr::from(addr.0)), port),
            Ipv6(ref addr) => SocketAddr::new(IpAddr::V6(Ipv6Addr::from(addr.0)), port),
            _ => {
                return Err( "peer_addr failed");
            }
        };

        Ok(saddr)
    }

    pub fn socket_addr(&self) -> IoResult<SocketAddr> {
        unsupported()
    }

    pub fn shutdown(&self, how: i32) -> IoResult<()> {
        tcpstream::shutdown(*self.0.as_inner(), how as i32)
            .map_err(|_|  "unable to shutdown socket")
    }

    pub fn duplicate(&self) -> IoResult<TcpStream> {
        Ok(self.clone())
    }

    pub fn set_linger(&self, _linger: Option<Duration>) -> IoResult<()> {
        unsupported()
    }

    pub fn linger(&self) -> IoResult<Option<Duration>> {
        unsupported()
    }

    pub fn set_nodelay(&self, mode: bool) -> IoResult<()> {
        tcpstream::set_nodelay(*self.0.as_inner(), mode)
            .map_err(|_|  "set_nodelay failed")
    }

    pub fn nodelay(&self) -> IoResult<bool> {
        tcpstream::nodelay(*self.0.as_inner())
            .map_err(|_|  "nodelay failed")
    }

    pub fn set_ttl(&self, tll: u32) -> IoResult<()> {
        tcpstream::set_tll(*self.0.as_inner(), tll)
            .map_err(|_|  "unable to set TTL")
    }

    pub fn ttl(&self) -> IoResult<u32> {
        tcpstream::get_tll(*self.0.as_inner())
            .map_err(|_|  "unable to get TTL")
    }

    pub fn take_error(&self) -> IoResult<()> {
        unsupported()
    }

    pub fn set_nonblocking(&self, mode: bool) -> IoResult<()> {
        tcpstream::set_nonblocking(*self.0.as_inner(), mode).map_err(|_| {
             "unable to set blocking mode"
        })
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
    pub fn bind(addr: IoResult<&SocketAddr>) -> IoResult<TcpListener> {
        let addr = addr?;

        Ok(TcpListener(*addr))
    }

    pub fn socket_addr(&self) -> IoResult<SocketAddr> {
        Ok(self.0)
    }

    pub fn accept(&self) -> IoResult<(TcpStream, SocketAddr)> {
        let (handle, ipaddr, port) = tcplistener::accept(self.0.port())
            .map_err(|_|  "accept failed")?;
        let saddr = match ipaddr {
            Ipv4(ref addr) => SocketAddr::new(IpAddr::V4(Ipv4Addr::from(addr.0)), port),
            Ipv6(ref addr) => SocketAddr::new(IpAddr::V6(Ipv6Addr::from(addr.0)), port),
            _ => {
                return Err( "accept failed");
            }
        };

        Ok((TcpStream(Arc::new(Socket(handle))), saddr))
    }

    pub fn duplicate(&self) -> IoResult<TcpListener> {
        Ok(self.clone())
    }

    pub fn set_ttl(&self, _: u32) -> IoResult<()> {
        unsupported()
    }

    pub fn ttl(&self) -> IoResult<u32> {
        unsupported()
    }

    pub fn set_only_v6(&self, _: bool) -> IoResult<()> {
        unsupported()
    }

    pub fn only_v6(&self) -> IoResult<bool> {
        unsupported()
    }

    pub fn take_error(&self) -> IoResult<()> {
        unsupported()
    }

    pub fn set_nonblocking(&self, _: bool) -> IoResult<()> {
        unsupported()
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

pub struct UdpSocket(Handle);

pub struct LookupHost(!);

impl LookupHost {
    pub fn port(&self) -> u16 {
        self.0
    }
}

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        self.0
    }
}

impl TryFrom<&str> for LookupHost {
    type Error = &'static str;

    fn try_from(_v: &str) -> IoResult<LookupHost> {
        unsupported()
    }
}

impl<'a> TryFrom<(&'a str, u16)> for LookupHost {
    type Error = &'static str;

    fn try_from(_v: (&'a str, u16)) -> IoResult<LookupHost> {
        unsupported()
    }
}

#[allow(nonstandard_style)]
pub mod netc {
    pub const AF_INET: u8 = 0;
    pub const AF_INET6: u8 = 1;
    pub type sa_family_t = u8;

    #[derive(Copy, Clone)]
    pub struct in_addr {
        pub s_addr: u32,
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in {
        pub sin_family: sa_family_t,
        pub sin_port: u16,
        pub sin_addr: in_addr,
    }

    #[derive(Copy, Clone)]
    pub struct in6_addr {
        pub s6_addr: [u8; 16],
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in6 {
        pub sin6_family: sa_family_t,
        pub sin6_port: u16,
        pub sin6_addr: in6_addr,
        pub sin6_flowinfo: u32,
        pub sin6_scope_id: u32,
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr {}

    pub type socklen_t = usize;
}
