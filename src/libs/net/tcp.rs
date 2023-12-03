use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};
use core::net::SocketAddr;

use smoltcp::iface;
use smoltcp::socket::tcp::{self, ConnectError, State};
// use smoltcp::time::Duration;
use smoltcp::wire::{IpEndpoint, IpListenEndpoint};

use crate::exported::shyperstd::io::PollState;
use crate::libs::error::ShyperError;
use crate::libs::net::addr::*;
use super::{now, network_poll};
use super::SmoltcpSocketHandle;
use super::interface::{get_ephemeral_port, check_local_endpoint};

// State transitions:
// CLOSED -(connect)-> BUSY -> CONNECTING -> CONNECTED -(shutdown)-> BUSY -> CLOSED
//       |
//       |-(listen)-> BUSY -> LISTENING -(shutdown)-> BUSY -> CLOSED
//       |
//        -(bind)-> BUSY -> CLOSED
const STATE_CLOSED: u8 = 0;
const STATE_BUSY: u8 = 1;
const STATE_CONNECTING: u8 = 2;
const STATE_CONNECTED: u8 = 3;
const STATE_LISTENING: u8 = 4;

#[derive(Debug)]
pub struct TcpSocket {
    state: AtomicU8,
    handle: UnsafeCell<SmoltcpSocketHandle>,
    local_addr: UnsafeCell<IpEndpoint>,
    peer_addr: UnsafeCell<IpEndpoint>,
    nonblocking: AtomicBool,
}

unsafe impl Sync for TcpSocket {}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        debug!("TcpSocket drop");
        self.close().expect("TcpSocket drop() close failed");
        super::NIC
            .lock()
            .as_nic_mut()
            .unwrap()
            .remove_tcp_handle(self.get_socket_handle());
    }
}

// impl Clone for TcpSocket {
//     fn clone(&self) -> Self {
//         let handle = super::NIC
//             .lock()
//             .as_nic_mut()
//             .unwrap()
//             .create_tcp_handle()
//             .unwrap();

//         Self {
//             state: AtomicU8::new(self.get_state()),
//             handle,
//             local_addr: UnsafeCell::new(unsafe { self.local_addr.get().read() }),
//             peer_addr: UnsafeCell::new(unsafe { self.peer_addr.get().read() }),
//             nonblocking: AtomicBool::new(self.is_nonblocking()),
//         }
//     }
// }

impl TcpSocket {
    pub fn new() -> Self {
        let handle = super::NIC
            .lock()
            .as_nic_mut()
            .unwrap()
            .create_tcp_handle()
            .unwrap();
        Self {
            state: AtomicU8::new(STATE_CLOSED),
            handle: UnsafeCell::new(handle),
            local_addr: UnsafeCell::new(UNSPECIFIED_ENDPOINT),
            peer_addr: UnsafeCell::new(UNSPECIFIED_ENDPOINT),
            nonblocking: AtomicBool::new(false),
        }
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, ShyperError> {
        Ok(ipendpoint_to_socketaddr(unsafe {
            self.peer_addr.get().read()
        }))
    }

    pub fn local_addr(&self) -> Result<SocketAddr, ShyperError> {
        Ok(ipendpoint_to_socketaddr(unsafe {
            self.local_addr.get().read()
        }))
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    #[inline]
    pub fn set_nonblocking(&self, mode: bool) -> Result<(), ShyperError> {
        self.nonblocking.store(mode, Ordering::Release);
        Ok(())
    }

    /// Returns whether this socket is in nonblocking mode.
    #[inline]
    pub fn is_nonblocking(&self) -> bool {
        self.nonblocking.load(Ordering::Acquire)
    }

    pub fn no_delay(&self) -> Result<bool, ShyperError> {
        self.with(|socket| {
            if !socket.is_active() {
                warn!("TcpSocket no_delay() socket is not actived");
                Err(ShyperError::ConnectionRefused)
            } else {
                Ok(socket.nagle_enabled())
            }
        })
    }

    pub fn set_no_delay(&self, mode: bool) -> Result<(), ShyperError> {
        self.with(|socket| {
            if !socket.is_active() {
                warn!("TcpSocket no_delay() socket is not actived");
                Err(ShyperError::ConnectionRefused)
            } else {
                socket.set_nagle_enabled(!mode);
                Ok(())
            }
        })
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// If the given port is 0, it generates one automatically.
    ///
    /// It's must be called before [`listen`](Self::listen) and
    /// [`accept`](Self::accept).
    pub fn bind(&self, mut local_addr: SocketAddr) -> Result<(), ShyperError> {
        debug!("TcpSocket bind at {:?}", local_addr);
        self.update_state(STATE_CLOSED, STATE_CLOSED, || {
            // TODO: check addr is available
            if local_addr.port() == 0 {
                local_addr.set_port(get_ephemeral_port()?);
            } else if check_local_endpoint(local_addr.port()) {
                warn!("socket bind() failed:, port has been occupied");
                return Err(ShyperError::AddrInUse);
            }
            // SAFETY: no other threads can read or write `self.local_addr` as we
            // have changed the state to `BUSY`.
            unsafe {
                let old = self.local_addr.get().read();
                if old != UNSPECIFIED_ENDPOINT {
                    warn!("socket bind() failed: already bound");
                    return Err(ShyperError::InvalidInput);
                }
                self.local_addr
                    .get()
                    .write(socketaddr_to_ipendpoint(local_addr));
            }
            Ok(())
        })
        .unwrap_or_else(|_| {
            warn!("socket bind() failed: already bound");
            Err(ShyperError::InvalidInput)
        })
    }

    /// Starts listening on the bound address and port.
    ///
    /// It's must be called after [`bind`](Self::bind) and before
    /// [`accept`](Self::accept).
    pub fn listen(&self) -> Result<(), ShyperError> {
        debug!("TcpSocket listen");
        self.update_state(STATE_CLOSED, STATE_LISTENING, || {
            let bound_endpoint = self.bound_endpoint()?;
            debug!("TcpSocket listen at {}", bound_endpoint);

            unsafe {
                (*self.local_addr.get()).port = bound_endpoint.port;
            }
            self.with(|socket| {
                if !socket.is_open() {
                    socket.listen(bound_endpoint).or_else(|e| {
                        warn!("socket listen() failed: ListenError {}", e);
                        Err(ShyperError::AddrInUse)
                    })
                } else {
                    warn!("socket listen() failed: socket is already open");
                    Err(ShyperError::AddrInUse)
                }
            })?;
            debug!("TCP socket listening on {}", bound_endpoint);
            Ok(())
        })
        .unwrap_or(Ok(())) // ignore simultaneous `listen`s.
    }

    pub fn connect(&self, remote_addr: SocketAddr) -> Result<(), ShyperError> {
        debug!("TcpSocket connect to {:?}", remote_addr);

        self.update_state(STATE_CLOSED, STATE_CONNECTING, || {
            let remote_endpoint = socketaddr_to_ipendpoint(remote_addr);
            let local_endpoint = self.bound_endpoint()?;
            debug!(
                "tcp_connect {} to remote {}, local {}",
                crate::libs::thread::current_thread_id(),
                remote_addr,
                local_endpoint
            );
            let (local_endpoint, remote_endpoint) = self.with_context(|socket, cx| {
                socket
                    .connect(cx, remote_endpoint, local_endpoint)
                    .or_else(|e| match e {
                        ConnectError::InvalidState => {
                            warn!("socket connect() failed on {}", e);
                            Err(ShyperError::BadState)
                        }
                        ConnectError::Unaddressable => {
                            warn!("socket connect() failed on {}", e);
                            Err(ShyperError::ConnectionRefused)
                        }
                    })?;
                Ok((
                    socket.local_endpoint().unwrap(),
                    socket.remote_endpoint().unwrap(),
                ))
            })?;
            unsafe {
                // SAFETY: no other threads can read or write these fields as we
                // have changed the state to `STATE_BUSY`.
                self.local_addr.get().write(local_endpoint);
                self.peer_addr.get().write(remote_endpoint);
            }
            Ok(())
        })
        .unwrap_or_else(|_| {
            warn!("socket connect() failed: already connected");
            Err(ShyperError::AlreadyExists)
        })?;

        // Here our state must be `CONNECTING`, and only one thread can run here.
        if self.is_nonblocking() {
            Err(ShyperError::WouldBlock)
        } else {
            self.block_on(|| {
                let PollState { writable, .. } = self.poll_connect()?;
                if !writable {
                    Err(ShyperError::WouldBlock)
                } else if self.get_state() == STATE_CONNECTED {
                    Ok(())
                } else {
                    warn!("socket connect() failed, bad state");
                    Err(ShyperError::ConnectionRefused)
                }
            })
        }
    }

    /// Accepts a new connection.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, a new [`TcpSocket`] is returned.
    ///
    /// It's must be called after [`bind`](Self::bind) and [`listen`](Self::listen).
    pub fn accept(&self) -> Result<Self, ShyperError> {
        if !self.is_listening() {
            warn!("socket accept() failed: not listen");
            return Err(ShyperError::InvalidInput);
        }

        debug!("TcpSocket accept()");

        self.block_on(|| {
            let mut guard = super::NIC.lock();
            let nic = guard.as_nic_mut().unwrap();

            let socket = nic.get_mut_socket::<tcp::Socket<'_>>(self.get_socket_handle());

            if socket.is_active() {
                let remote_endpoint = socket.remote_endpoint().unwrap();
                let local_endpoint = socket.local_endpoint().unwrap();
                // let _ = drop(socket);

                debug!(
                    "TcpSocket accept local {}, remote {}",
                    local_endpoint, remote_endpoint
                );

                // Create new SocketHandle and listen for next socket connect.
                let new_handle = nic.create_tcp_handle().unwrap();
                nic.get_mut_socket::<tcp::Socket<'_>>(new_handle)
                    .listen(local_endpoint)
                    .expect("socket accept() failed: newly created SocketHandle listen failed");

                // Replace self SocketHandle with newly created SocketHandle.
                let old_handle = unsafe { self.handle.get().replace(new_handle) };

                // Return connected TcpSocket with ori SocketHandle.
                let new_socket =
                    TcpSocket::new_connected(old_handle, local_endpoint, remote_endpoint);

                // To flush send buffers.
                // After using the socket, the network interface has to poll the nic,
                // This is required to flush all send buffers.
                nic.poll_common(now());

                Ok(new_socket)
            } else {
                match socket.state() {
                    State::Closed | State::Closing | State::FinWait1 | State::FinWait2 => {
                        warn!(
                            "socket accept() failed: state {} mis-matched",
                            socket.state()
                        );
                        Err(ShyperError::Io)
                    }
                    _ => Err(ShyperError::WouldBlock),
                }
            }
        })
    }

    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, ShyperError> {
        debug!("TcpSocket read()");

        if self.is_connecting() {
            warn!("TcpSocket write() failed on STATE_CONNECTING");
            return Err(ShyperError::WouldBlock);
        } else if !self.is_connected() {
            warn!("TcpSocket read() failed");
            return Err(ShyperError::NotConnected);
        }

        self.block_on(|| {
            self.with(|socket| {
                if !socket.is_active() {
                    warn!("TcpSocket read() socket is not actived");
                    Err(ShyperError::ConnectionRefused)
                } else if socket.can_recv() {
                    socket
                        .recv(|data| {
                            let len = core::cmp::min(buffer.len(), data.len());
                            buffer[..len].copy_from_slice(&data[..len]);
                            (len, len)
                        })
                        .map_err(|e| {
                            warn!("TcpSocket read() error on {}", e);
                            ShyperError::WouldBlock
                        })
                } else {
                    // socket.register_recv_waker(cx.waker());
                    Err(ShyperError::WouldBlock)
                }
            })
        })
    }

    pub fn write(&self, buffer: &[u8]) -> Result<usize, ShyperError> {
        debug!("TcpSocket write() {} bytes", buffer.len());

        if self.is_connecting() {
            warn!("TcpSocket write() failed on STATE_CONNECTING");
            return Err(ShyperError::WouldBlock);
        } else if !self.is_connected() {
            warn!("TcpSocket write() failed not connected");
            return Err(ShyperError::NotConnected);
        }

        let mut pos: usize = 0;

        while pos < buffer.len() {
            let n = self.block_on(|| {
                self.with(|socket| {
                    if !socket.is_active() {
                        warn!("TcpSocket write() socket is not actived");
                        Err(ShyperError::ConnectionRefused)
                    } else if socket.can_send() {
                        socket.send_slice(&buffer[pos..]).map_err(|e| {
                            warn!("TcpSocket write() error on {}", e);
                            ShyperError::BadState
                        })
                    } else if pos > 0 {
                        // we already send some data => return 0 as signal to stop the
                        // async write
                        Ok(0)
                    } else {
                        Err(ShyperError::WouldBlock)
                    }
                })
            })?;

            if n == 0 {
                return Ok(pos);
            }

            pos += n;
        }

        Ok(pos)
    }

    pub fn close(&self) -> Result<(), ShyperError> {
        debug!("TcpSocket close()");

        self.block_on(|| {
            self.with(|socket| match socket.state() {
                State::FinWait1
                | State::FinWait2
                | State::Closed
                | State::Closing
                | State::TimeWait => Err(ShyperError::BadState),
                _ => {
                    if socket.send_queue() > 0 {
                        // socket.register_send_waker(cx.waker());
                        Err(ShyperError::WouldBlock)
                    } else {
                        socket.close();
                        Ok(())
                    }
                }
            })
        })?;

        self.block_on(|| {
            self.with(|socket| match socket.state() {
                State::FinWait1
                | State::FinWait2
                | State::Closed
                | State::Closing
                | State::TimeWait => Ok(()),
                _ => {
                    // socket.register_send_waker(cx.waker());
                    Err(ShyperError::WouldBlock)
                }
            })
        })
    }

    /// Whether the socket is readable or writable.
    pub fn poll(&self) -> Result<PollState, ShyperError> {
        match self.get_state() {
            STATE_CONNECTING => self.poll_connect(),
            STATE_CONNECTED => self.poll_stream(),
            // This is different from ArceOS's listen_table design.
            // STATE_LISTENING => self.poll_listener(),
            _ => Ok(PollState {
                readable: false,
                writable: false,
            }),
        }
    }
}

/// Private methods
impl TcpSocket {
    /// Creates a new TCP socket that is already connected.
    const fn new_connected(
        handle: SmoltcpSocketHandle,
        local_addr: IpEndpoint,
        peer_addr: IpEndpoint,
    ) -> Self {
        Self {
            state: AtomicU8::new(STATE_CONNECTED),
            handle: UnsafeCell::new(handle),
            local_addr: UnsafeCell::new(local_addr),
            peer_addr: UnsafeCell::new(peer_addr),
            nonblocking: AtomicBool::new(false),
        }
    }

    #[inline]
    fn get_socket_handle(&self) -> SmoltcpSocketHandle {
        unsafe { self.handle.get().read() }
    }

    #[inline]
    fn get_state(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }

    #[inline]
    fn set_state(&self, state: u8) {
        self.state.store(state, Ordering::Release);
    }

    /// Update the state of the socket atomically.
    ///
    /// If the current state is `expect`, it first changes the state to `STATE_BUSY`,
    /// then calls the given function. If the function returns `Ok`, it changes the
    /// state to `new`, otherwise it changes the state back to `expect`.
    ///
    /// It returns `Ok` if the current state is `expect`, otherwise it returns
    /// the current state in `Err`.
    fn update_state<F, T>(&self, expect: u8, new: u8, f: F) -> Result<Result<T, ShyperError>, u8>
    where
        F: FnOnce() -> Result<T, ShyperError>,
    {
        match self
            .state
            .compare_exchange(expect, STATE_BUSY, Ordering::Acquire, Ordering::Acquire)
        {
            Ok(_) => {
                let res = f();
                if res.is_ok() {
                    self.set_state(new);
                } else {
                    self.set_state(expect);
                }
                Ok(res)
            }
            Err(old) => Err(old),
        }
    }

    fn with<R>(&self, f: impl FnOnce(&mut tcp::Socket<'_>) -> R) -> R {
        let mut guard = super::NIC.lock();
        let nic = guard.as_nic_mut().unwrap();
        let result = f(nic.get_mut_socket::<tcp::Socket<'_>>(self.get_socket_handle()));
        // To flush send buffers.
        // After using the socket, the network interface has to poll the nic,
        // This is required to flush all send buffers.
        nic.poll_common(now());
        result
    }

    fn with_context<R>(&self, f: impl FnOnce(&mut tcp::Socket<'_>, &mut iface::Context) -> R) -> R {
        let mut guard = super::NIC.lock();
        let nic = guard.as_nic_mut().unwrap();
        let res = {
            let (s, cx) = nic.get_socket_and_context::<tcp::Socket<'_>>(self.get_socket_handle());
            f(s, cx)
        };
        nic.poll_common(now());
        res
    }

    /// Block the current thread until the given function completes or fails.
    ///
    /// If the socket is non-blocking, it calls the function once and returns
    /// immediately. Otherwise, it may call the function multiple times if it
    /// returns [`Err(WouldBlock)`](AxError::WouldBlock).
    fn block_on<F, T>(&self, mut f: F) -> Result<T, ShyperError>
    where
        F: FnMut() -> Result<T, ShyperError>,
    {
        if self.is_nonblocking() {
            f()
        } else {
            loop {
                network_poll();
                match f() {
                    Ok(t) => return Ok(t),
                    Err(ShyperError::WouldBlock) => crate::libs::thread::thread_yield(),
                    Err(e) => return Err(e),
                }
            }
        }
    }

    #[inline]
    fn is_connecting(&self) -> bool {
        self.get_state() == STATE_CONNECTING
    }

    #[inline]
    fn is_connected(&self) -> bool {
        self.get_state() == STATE_CONNECTED
    }

    #[inline]
    fn is_listening(&self) -> bool {
        self.get_state() == STATE_LISTENING
    }

    fn bound_endpoint(&self) -> Result<IpListenEndpoint, ShyperError> {
        // SAFETY: no other threads can read or write `self.local_addr`.
        let local_addr = unsafe { self.local_addr.get().read() };
        let port = if local_addr.port != 0 {
            local_addr.port
        } else {
            get_ephemeral_port()?
        };
        assert_ne!(port, 0);
        let addr = if !is_unspecified(local_addr.addr) {
            Some(local_addr.addr)
        } else {
            None
        };
        Ok(IpListenEndpoint { addr, port })
    }

    fn poll_connect(&self) -> Result<PollState, ShyperError> {
        let writable = self.with(|socket| match socket.state() {
            State::SynSent => false, // wait for connection
            State::Established => {
                self.set_state(STATE_CONNECTED); // connected
                debug!(
                    "TCP socket {}: connected to {}",
                    socket.local_endpoint().unwrap(),
                    socket.remote_endpoint().unwrap(),
                );
                true
            }
            _ => {
                unsafe {
                    self.local_addr.get().write(UNSPECIFIED_ENDPOINT);
                    self.peer_addr.get().write(UNSPECIFIED_ENDPOINT);
                }
                self.set_state(STATE_CLOSED); // connection failed
                true
            }
        });
        Ok(PollState {
            readable: false,
            writable,
        })
    }

    fn poll_stream(&self) -> Result<PollState, ShyperError> {
        self.with(|socket| {
            Ok(PollState {
                readable: !socket.may_recv() || socket.can_recv(),
                writable: !socket.may_send() || socket.can_send(),
            })
        })
    }
}

/// Possible values which can be passed to the [`TcpStream::shutdown`] method.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Shutdown {
    /// The reading portion of the [`TcpStream`] should be shut down.
    ///
    /// All currently blocked and future [reads] will return <code>[Ok]\(0)</code>.
    ///
    /// [reads]: crate::io::Read "io::Read"
    Read,
    /// The writing portion of the [`TcpStream`] should be shut down.
    ///
    /// All currently blocked and future [writes] will return an error.
    ///
    /// [writes]: crate::io::Write "io::Write"
    Write,
    /// Both the reading and the writing portions of the [`TcpStream`] should be shut down.
    ///
    /// See [`Shutdown::Read`] and [`Shutdown::Write`] for more information.
    Both,
}

impl Shutdown {
    pub fn from_i32(value: i32) -> Shutdown {
        match value {
            0 => Shutdown::Read,
            1 => Shutdown::Write,
            2 => Shutdown::Both,
            _ => panic!("Unknown value: {}", value),
        }
    }
}
