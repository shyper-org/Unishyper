use core::task::Poll;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use alloc::boxed::Box;
use core::net::SocketAddr;

use futures_lite::future;
use smoltcp::iface;
use smoltcp::socket::tcp::{self, ConnectError};
use smoltcp::time::Duration;
use smoltcp::wire::{IpAddress, IpEndpoint};

use crate::libs::error::ShyperError;
use crate::libs::net::addr::*;
use super::now;
use super::SmoltcpSocketHandle;
use super::Handle;

#[derive(Debug)]
pub struct AsyncTcpSocket {
    handle: SmoltcpSocketHandle,
    port: AtomicU16,
    local_addr: UnsafeCell<IpEndpoint>,
    peer_addr: UnsafeCell<IpEndpoint>,
    nonblocking: AtomicBool,
}

/// Todo: This interface seems awkward.
/// We need to find a new way to handle the relationship between `AsyncTcpSocket` and `Handle`.
impl From<Handle> for Box<AsyncTcpSocket> {
    fn from(handle: Handle) -> Self {
        let ptr = handle.0;
        unsafe { Box::from_raw(ptr as *mut AsyncTcpSocket) }
    }
}

impl Drop for AsyncTcpSocket {
    fn drop(&mut self) {
        debug!("AsyncTcpSocket drop");
        // self.close()
    }
}

impl AsyncTcpSocket {
    pub fn new(port: u16) -> Self {
        let handle = super::NIC
            .lock()
            .as_nic_mut()
            .unwrap()
            .create_tcp_handle()
            .unwrap();
        Self {
            handle,
            port: AtomicU16::new(port),
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

    fn with<R>(&self, f: impl FnOnce(&mut tcp::Socket<'_>) -> R) -> R {
        let mut guard = super::NIC.lock();
        let nic = guard.as_nic_mut().unwrap();
        let result = f(nic.get_mut_socket::<tcp::Socket<'_>>(self.handle));
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
            let (s, cx) = nic.get_socket_and_context::<tcp::Socket<'_>>(self.handle);
            f(s, cx)
        };
        nic.poll_common(now());
        res
    }

    pub async fn connect(
        &self,
        address: IpAddress,
        port: u16,
        local_endpoint: u16,
    ) -> Result<SmoltcpSocketHandle, ShyperError> {
        debug!(
            "tcp_connect {} to ip {}:{}, local_endpoint {}",
            crate::libs::thread::current_thread_id(),
            address,
            port,
            local_endpoint
        );
        self.with_context(|socket, cx| {
            socket
                .connect(
                    cx,
                    (address, port),
                    local_endpoint,
                    // LOCAL_ENDPOINT.fetch_add(1, Ordering::SeqCst),
                )
                .or_else(|e| match e {
                    ConnectError::InvalidState => {
                        warn!("socket connect() failed on {}", e);
                        Err(ShyperError::BadState)
                    }
                    ConnectError::Unaddressable => {
                        warn!("socket connect() failed on {}", e);
                        Err(ShyperError::ConnectionRefused)
                    }
                })
        })?;

        future::poll_fn(|cx| {
            self.with(|socket| match socket.state() {
                tcp::State::Closed | tcp::State::TimeWait => {
                    Poll::Ready(Err(ShyperError::BadAddress))
                }
                tcp::State::Listen => Poll::Ready(Err(ShyperError::BadState)),
                tcp::State::SynSent | tcp::State::SynReceived => {
                    socket.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready({
                    unsafe {
                        self.local_addr
                            .get()
                            .write(socket.local_endpoint().unwrap());
                        self.peer_addr
                            .get()
                            .write(socket.remote_endpoint().unwrap());
                    }

                    Ok(self.handle)
                }),
            })
        })
        .await
    }

    pub async fn accept(&self) -> Result<(), ShyperError> {
        future::poll_fn(|cx| {
            self.with(|socket| match socket.state() {
                tcp::State::Closed => {
                    let _ = socket.listen(self.port.load(Ordering::Acquire));
                    Poll::Ready(())
                }
                tcp::State::Listen => {
                    debug!("Socket accept success!!");
                    Poll::Ready(())
                }
                _ => {
                    socket.register_recv_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await;

        future::poll_fn(|cx| {
            self.with(|socket| {
                if socket.is_active() {
                    unsafe {
                        self.local_addr
                            .get()
                            .write(socket.local_endpoint().unwrap());
                        self.peer_addr
                            .get()
                            .write(socket.remote_endpoint().unwrap());
                    }
                    Poll::Ready(Ok(()))
                } else {
                    match socket.state() {
                        tcp::State::Closed
                        | tcp::State::Closing
                        | tcp::State::FinWait1
                        | tcp::State::FinWait2 => Poll::Ready(Err(ShyperError::Io)),
                        _ => {
                            socket.register_recv_waker(cx.waker());
                            Poll::Pending
                        }
                    }
                }
            })
        })
        .await?;

        let mut guard = super::NIC.lock();
        let nic = guard.as_nic_mut()?;
        let socket = nic.get_mut_socket::<tcp::Socket<'_>>(self.handle);
        socket.set_keep_alive(Some(Duration::from_millis(
            super::DEFAULT_KEEP_ALIVE_INTERVAL,
        )));

        Ok(())
    }

    pub async fn read(&self, buffer: &mut [u8]) -> Result<usize, ShyperError> {
        future::poll_fn(|cx| {
            self.with(|socket| {
                if !socket.is_active() {
                    warn!("TcpSocket read() socket is not actived");
                    Poll::Ready(Err(ShyperError::ConnectionRefused))
                } else if socket.can_recv() {
                    Poll::Ready(
                        socket
                            .recv(|data| {
                                let len = core::cmp::min(buffer.len(), data.len());
                                buffer[..len].copy_from_slice(&data[..len]);
                                (len, len)
                            })
                            .map_err(|e| {
                                warn!("TcpSocket read() error on {}", e);
                                ShyperError::BadState
                            }),
                    )
                } else {
                    socket.register_recv_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await
    }

    pub async fn write(&self, buffer: &[u8]) -> Result<usize, ShyperError> {
        let mut pos: usize = 0;

        while pos < buffer.len() {
            let n = future::poll_fn(|cx| {
                self.with(|socket| {
                    if !socket.is_active() {
                        warn!("TcpSocket write() socket is not actived");
                        Poll::Ready(Err(ShyperError::ConnectionRefused))
                    } else if socket.can_send() {
                        Poll::Ready(socket.send_slice(&buffer[pos..]).map_err(|e| {
                            warn!("TcpSocket write() error on {}", e);
                            ShyperError::BadState
                        }))
                    } else if pos > 0 {
                        // we already send some data => return 0 as signal to stop the
                        // async write
                        Poll::Ready(Ok(0))
                    } else {
                        socket.register_send_waker(cx.waker());
                        Poll::Pending
                    }
                })
            })
            .await?;

            if n == 0 {
                return Ok(pos);
            }

            pos += n;
        }

        Ok(pos)
    }

    pub async fn close(&self) -> Result<(), ShyperError> {
        future::poll_fn(|cx| {
            self.with(|socket| match socket.state() {
                tcp::State::FinWait1
                | tcp::State::FinWait2
                | tcp::State::Closed
                | tcp::State::Closing
                | tcp::State::TimeWait => Poll::Ready(Err(ShyperError::BadState)),
                _ => {
                    if socket.send_queue() > 0 {
                        socket.register_send_waker(cx.waker());
                        Poll::Pending
                    } else {
                        socket.close();
                        Poll::Ready(Ok(()))
                    }
                }
            })
        })
        .await?;

        future::poll_fn(|cx| {
            self.with(|socket| match socket.state() {
                tcp::State::FinWait1
                | tcp::State::FinWait2
                | tcp::State::Closed
                | tcp::State::Closing
                | tcp::State::TimeWait => Poll::Ready(Ok(())),
                _ => {
                    socket.register_send_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await
    }
}

// extern "C" fn nic_thread(_: usize) {
//     info!("[nic_thread] Enter super::NIC thread\n*********************************************\n");
//     loop {
//         debug!("[nic_thread] enter netwait");

//         netwait();

//         debug!("[nic_thread] netwait finished, try to call nic.poll_common");

//         if let NetworkState::Initialized(nic) = super::NIC.lock().deref_mut() {
//             // debug!("NetworkState Initialized success, poll_common");
//             nic.poll_common(Instant::from_millis(current_ms() as i64));
//             nic.wake();
//         }
//     }
// }

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
