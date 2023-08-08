use core::sync::atomic::{AtomicBool, Ordering};
use spin::RwLock;

use no_std_net::SocketAddr;
use smoltcp::wire::IpEndpoint;
use smoltcp::socket::UdpSocket;
use smoltcp::Error;

use crate::libs::error::ShyperError;

use super::Handle;
use super::addr::*;

pub struct AsyncUdpSocket {
    handle: Handle,
    local_addr: RwLock<Option<IpEndpoint>>,
    peer_addr: RwLock<Option<IpEndpoint>>,
    nonblock: AtomicBool,
}

#[allow(unused)]
impl AsyncUdpSocket {
    pub fn new() -> Self {
        let handle = super::NIC
            .lock()
            .as_nic_mut()
            .unwrap()
            .create_udp_handle()
            .unwrap();
        // println!("create handle {:?}",handle);
        Self {
            handle,
            local_addr: RwLock::new(None),
            peer_addr: RwLock::new(None),
            nonblock: AtomicBool::new(false),
        }
    }

    fn with<R>(&self, f: impl FnOnce(&mut UdpSocket<'_>) -> R) -> R {
        // println!("Async Socket with(), handle {:?}", self.0);

        let mut guard = super::NIC.lock();
        let nic = guard.as_nic_mut().unwrap();
        let res = {
            let s = nic.iface.get_socket::<UdpSocket<'_>>(self.handle);
            let res = f(s);
            res
        };
        // To flush send buffers.
        // After using the socket, the network interface has to poll the nic,
        // This is required to flush all send buffers.
        let t = super::now();
        if nic.poll_delay(t).map(|d| d.total_millis()).unwrap_or(0) == 0 {
            nic.poll_common(t);
        }
        res
    }

    /// Returns whether this socket is in nonblocking mode.
    #[inline]
    pub fn is_nonblocking(&self) -> bool {
        self.nonblock.load(Ordering::Acquire)
    }

    /// Moves this UDP socket into or out of nonblocking mode.
    #[inline]
    pub fn set_nonblocking(&self, nonblocking: bool) {
        self.nonblock.store(nonblocking, Ordering::Release);
    }

    pub fn local_addr(&self) -> Result<SocketAddr, ShyperError> {
        match self.local_addr.try_read() {
            Some(addr) => addr
                .map(ipendpoint_to_core_socketaddr)
                .ok_or(ShyperError::NotConnected),
            None => Err(ShyperError::NotConnected),
        }
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, ShyperError> {
        self.remote_endpoint().map(ipendpoint_to_core_socketaddr)
    }

    pub fn bind(&self, mut local_addr: SocketAddr) -> Result<(), ShyperError> {
        let mut self_local_addr = self.local_addr.write();

        if local_addr.port() == 0 {
            local_addr.set_port(super::interface::get_local_endpoint());
        }
        if self_local_addr.is_some() {
            warn!("UdpSocket bind() failed: already bound");
            return Err(ShyperError::InvalidInput);
        }

        let local_endpoint = core_socketaddr_to_ipendpoint(local_addr);
        // debug!("local endpoint {:?}", local_endpoint);
        // let endpoint = IpEndpoint {
        //     addr: local_endpoint.addr,
        //     port: local_endpoint.port,
        // };
        self.with(|socket| socket.bind(local_endpoint))
            .map_err(|e| {
                warn!("UdpSocket bind() failed: error {}", e);
                ShyperError::InvalidInput
            })?;

        *self_local_addr = Some(local_endpoint);
        debug!("UDP socket {}: bound on {}", self.handle, local_endpoint);
        Ok(())
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    pub fn send_to(&self, buf: &[u8], remote_addr: SocketAddr) -> Result<usize, ShyperError> {
        if remote_addr.port() == 0 || remote_addr.ip().is_unspecified() {
            return Err(ShyperError::InvalidInput);
            // return ax_err!(InvalidInput, "socket send_to() failed: invalid address");
        }
        self.send_impl(buf, core_socketaddr_to_ipendpoint(remote_addr))
    }

    /// Receives a single datagram message on the socket. On success, returns
    /// the number of bytes read and the origin.
    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), ShyperError> {
        self.recv_impl(|socket| match socket.recv_slice(buf) {
            Ok((len, endpoint)) => Ok((len, ipendpoint_to_core_socketaddr(endpoint))),
            Err(err) => {
                warn!("AsyncUdpsocket recv_from() failed on err {}", err);
                Err(ShyperError::BadState)
            }
        })
    }

    /// Receives a single datagram message on the socket, without removing it from
    /// the queue. On success, returns the number of bytes read and the origin.
    pub fn peek_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), ShyperError> {
        self.recv_impl(|socket| match socket.peek_slice(buf) {
            Ok((len, endpoint)) => Ok((len, ipendpoint_to_core_socketaddr(*endpoint))),
            Err(err) => {
                warn!("AsyncUdpsocket recv_from() failed on err {}", err);
                Err(ShyperError::BadState)
            }
        })
    }

    /// Connects this UDP socket to a remote address, allowing the `send` and
    /// `recv` to be used to send data and also applies filters to only receive
    /// data from the specified address.
    ///
    /// The local port will be generated automatically if the socket is not bound.
    /// It's must be called before [`send`](Self::send) and
    /// [`recv`](Self::recv).
    pub fn connect(&self, addr: SocketAddr) -> Result<(), ShyperError> {
        debug!("UDP socket connect to addr {:?}", addr);
        let mut self_peer_addr = self.peer_addr.write();

        if self.local_addr.read().is_none() {
            self.bind(ipendpoint_to_core_socketaddr(UNSPECIFIED_ENDPOINT))?;
        }

        *self_peer_addr = Some(core_socketaddr_to_ipendpoint(addr));
        debug!("UDP socket {}: connected to {}", self.handle, addr);
        Ok(())
    }

    /// Sends data on the socket to the remote address to which it is connected.
    pub fn send(&self, buf: &[u8]) -> Result<usize, ShyperError> {
        let remote_endpoint = self.remote_endpoint()?;
        self.send_impl(buf, remote_endpoint)
    }

    /// Receives a single datagram message on the socket from the remote address
    /// to which it is connected. On success, returns the number of bytes read.
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize, ShyperError> {
        let remote_endpoint = self.remote_endpoint()?;
        self.recv_impl(|socket| {
            let (len, endpoint) = socket.recv_slice(buf).map_err(|err| {
                warn!("AsyncUdpSocket recv() failed on err {}", err);
                ShyperError::BadState
            })?;
            if !is_unspecified(remote_endpoint.addr) && remote_endpoint.addr != endpoint.addr {
                return Err(ShyperError::WouldBlock);
            }
            if remote_endpoint.port != 0 && remote_endpoint.port != endpoint.port {
                return Err(ShyperError::WouldBlock);
            }
            Ok(len)
        })
    }

    /// Close the socket.
    pub fn shutdown(&self) -> Result<(), ShyperError> {
        self.with(|socket| socket.close());
        Ok(())
    }

    // Whether the socket is readable or writable.
    // pub fn poll(&self) -> Result<PollState> {
    //     if self.local_addr.read().is_none() {
    //         return Ok(PollState {
    //             readable: false,
    //             writable: false,
    //         });
    //     }
    //     SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
    //         Ok(PollState {
    //             readable: socket.can_recv(),
    //             writable: socket.can_send(),
    //         })
    //     })
    // }
}

/// Private methods
impl AsyncUdpSocket {
    fn remote_endpoint(&self) -> Result<IpEndpoint, ShyperError> {
        match self.peer_addr.try_read() {
            Some(addr) => addr.ok_or(ShyperError::NotConnected),
            None => Err(ShyperError::NotConnected),
        }
    }

    fn send_impl(&self, buf: &[u8], remote_endpoint: IpEndpoint) -> Result<usize, ShyperError> {
        if self.local_addr.read().is_none() {
            warn!("UdpSocket send() failed, no local addr");
            return Err(ShyperError::NotConnected);
        }

        self.block_on(|| {
            self.with(|socket| {
                if socket.can_send() {
                    socket
                        .send_slice(buf, remote_endpoint)
                        .map_err(|e| match e {
                            Error::Exhausted => ShyperError::WouldBlock,
                            _ => {
                                warn!("UdpSocket send() failed on error {}", e);
                                ShyperError::ConnectionRefused
                            }
                        })?;
                    Ok(buf.len())
                } else {
                    // no more data
                    Err(ShyperError::WouldBlock)
                }
            })
        })
    }

    fn recv_impl<F, T>(&self, mut op: F) -> Result<T, ShyperError>
    where
        F: FnMut(&mut UdpSocket) -> Result<T, ShyperError>,
    {
        if self.local_addr.read().is_none() {
            warn!("UdpSocket receive() failed, no local addr");
            return Err(ShyperError::NotConnected);
        }

        self.block_on(|| {
            self.with(|socket| {
                if socket.can_recv() {
                    // data available
                    op(socket)
                } else {
                    // no more data
                    Err(ShyperError::WouldBlock)
                }
            })
        })
    }

    fn block_on<F, T>(&self, mut f: F) -> Result<T, ShyperError>
    where
        F: FnMut() -> Result<T, ShyperError>,
    {
        if self.is_nonblocking() {
            f()
        } else {
            loop {
                super::network_poll();
                match f() {
                    Ok(t) => return Ok(t),
                    Err(ShyperError::WouldBlock) => crate::libs::thread::thread_yield(),
                    Err(e) => return Err(e),
                }
            }
        }
    }
}