use alloc::str::FromStr;
use alloc::str;
use core::task::Poll;

use futures_lite::future;
use smoltcp::iface;
use smoltcp::socket::{TcpSocket, TcpState};
use smoltcp::time::Duration;
use smoltcp::wire::IpAddress;
use smoltcp::wire::IpEndpoint;
use smoltcp::Error;

use super::now;
use super::executor::block_on;
use super::Handle;

/// Default keep alive interval in milliseconds
const DEFAULT_KEEP_ALIVE_INTERVAL: u64 = 75000;

pub struct AsyncTcpSocket(Handle);

impl AsyncTcpSocket {
    pub fn new() -> Self {
        let handle = super::NIC
            .lock()
            .as_nic_mut()
            .unwrap()
            .create_tcp_handle()
            .unwrap();
        // println!("create handle {:?}",handle);
        Self(handle)
    }

    pub(crate) fn inner(&self) -> Handle {
        self.0
    }

    fn with<R>(&self, f: impl FnOnce(&mut TcpSocket<'_>) -> R) -> R {
        // println!("Async Socket with(), handle {:?}", self.0);

        let mut guard = super::NIC.lock();
        let nic = guard.as_nic_mut().unwrap();
        let res = {
            let s = nic.iface.get_socket::<TcpSocket<'_>>(self.0);
            let res = f(s);
            res
        };
        // To flush send buffers.
        // After using the socket, the network interface has to poll the nic,
        // This is required to flush all send buffers.
        let t = now();
        if nic.poll_delay(t).map(|d| d.total_millis()).unwrap_or(0) == 0 {
            nic.poll_common(t);
        }
        res
    }

    fn with_context<R>(
        &self,
        f: impl FnOnce(&mut TcpSocket<'_>, &mut iface::Context<'_>) -> R,
    ) -> R {
        let mut guard = super::NIC.lock();
        let nic = guard.as_nic_mut().unwrap();
        let res = {
            let (s, cx) = nic.iface.get_socket_and_context::<TcpSocket<'_>>(self.0);
            f(s, cx)
        };
        // To flush send buffers.
        // After using the socket, the network interface has to poll the nic,
        // This is required to flush all send buffers.
        let t = now();
        if nic.poll_delay(t).map(|d| d.total_millis()).unwrap_or(0) == 0 {
            nic.poll_common(t);
        }
        res
    }

    pub async fn connect(
        &self,
        address: IpAddress,
        port: u16,
        local_endpoint: u16,
    ) -> Result<Handle, Error> {
        debug!(
            "tcp_stream_connect T[{}] to ip {}:{}, local_endpoint {}",
            crate::libs::thread::current_thread_id(),
            address,
            port,
            local_endpoint
        );
        self.with_context(|socket, cx| {
            socket.connect(
                cx,
                (address, port),
                local_endpoint,
                // LOCAL_ENDPOINT.fetch_add(1, Ordering::SeqCst),
            )
        })
        .map_err(|_| Error::Illegal)?;

        future::poll_fn(|cx| {
            self.with(|socket| match socket.state() {
                TcpState::Closed | TcpState::TimeWait => Poll::Ready(Err(Error::Unaddressable)),
                TcpState::Listen => Poll::Ready(Err(Error::Illegal)),
                TcpState::SynSent | TcpState::SynReceived => {
                    socket.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(self.0)),
            })
        })
        .await
    }

    pub async fn accept(&self, port: u16) -> Result<(IpAddress, u16), Error> {
        debug!("AsyncTcpSocket accept");
        self.with(|socket| socket.listen(port).map_err(|_| Error::Illegal))?;

        future::poll_fn(|cx| {
            self.with(|socket| {
                if socket.is_active() {
                    debug!("AsyncTcpSocket is_active state {}", socket.state());
                    Poll::Ready(Ok(()))
                } else {
                    match socket.state() {
                        TcpState::Closed
                        | TcpState::Closing
                        | TcpState::FinWait1
                        | TcpState::FinWait2 => Poll::Ready(Err(Error::Illegal)),
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
        let nic = guard.as_nic_mut().map_err(|_| Error::Illegal)?;
        let socket = nic.iface.get_socket::<TcpSocket<'_>>(self.0);
        debug!("AsyncTcpSocket accept state {}", socket.state());
        socket.set_keep_alive(Some(Duration::from_millis(DEFAULT_KEEP_ALIVE_INTERVAL)));
        let endpoint = socket.remote_endpoint();

        Ok((endpoint.addr, endpoint.port))
    }

    pub async fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        future::poll_fn(|cx| {
            self.with(|socket| {
                if !socket.is_active() {
                    Poll::Ready(Err(Error::Illegal))
                } else if socket.can_recv() {
                    Poll::Ready(
                        socket
                            .recv(|data| {
                                let len = core::cmp::min(buffer.len(), data.len());
                                buffer[..len].copy_from_slice(&data[..len]);
                                (len, len)
                            })
                            .map_err(|_| Error::Illegal),
                    )
                } else {
                    socket.register_recv_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await
    }

    pub async fn write(&self, buffer: &[u8]) -> Result<usize, Error> {
        let mut pos: usize = 0;

        while pos < buffer.len() {
            let n = future::poll_fn(|cx| {
                self.with(|socket| {
                    if !socket.is_active() {
                        warn!("socket is not actived");
                        Poll::Ready(Err(Error::Illegal))
                    } else if socket.can_send() {
                        Poll::Ready(
                            socket
                                .send_slice(&buffer[pos..])
                                .map_err(|_| Error::Illegal),
                        )
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

    pub async fn close(&self) -> Result<(), Error> {
        future::poll_fn(|cx| {
            self.with(|socket| match socket.state() {
                TcpState::FinWait1
                | TcpState::FinWait2
                | TcpState::Closed
                | TcpState::Closing
                | TcpState::TimeWait => Poll::Ready(Err(Error::Illegal)),
                _ => {
                    if socket.send_queue() > 0 {
                        socket.register_send_waker(cx.waker());
                        Poll::Pending
                    } else {
                        socket.close();
                        super::interface::remove_local_endpoint(socket.local_endpoint().port);
                        Poll::Ready(Ok(()))
                    }
                }
            })
        })
        .await?;

        future::poll_fn(|cx| {
            self.with(|socket| match socket.state() {
                TcpState::FinWait1
                | TcpState::FinWait2
                | TcpState::Closed
                | TcpState::Closing
                | TcpState::TimeWait => Poll::Ready(Ok(())),
                _ => {
                    socket.register_send_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await
    }
}

impl From<Handle> for AsyncTcpSocket {
    fn from(handle: Handle) -> Self {
        AsyncTcpSocket(handle)
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

/// Opens a TCP connection to a remote host.
#[inline(always)]
pub fn tcp_stream_connect(ip: &[u8], port: u16, timeout: Option<u64>) -> Result<Handle, ()> {
    let socket = AsyncTcpSocket::new();
    let local_endpoint = super::interface::get_local_endpoint();
    let address = IpAddress::from_str(str::from_utf8(ip).map_err(|_| ())?).map_err(|_| ())?;
    debug!(
        "tcp_stream_connect T[{}] to {}:{}",
        crate::libs::thread::current_thread_id(),
        address,
        port
    );
    let res = block_on(
        socket.connect(address, port, local_endpoint),
        timeout.map(Duration::from_millis),
    )?
    .map_err(|_| ());
    debug!(
        "tcp_stream_connect T[{}] to {}:{} success local_endpoint {}",
        crate::libs::thread::current_thread_id(),
        address,
        port,
        local_endpoint
    );
    super::interface::set_local_endpoint_link(local_endpoint, IpEndpoint::new(address, port));
    res
}

#[inline(always)]
pub fn tcp_stream_read(handle: Handle, buffer: &mut [u8]) -> Result<usize, ()> {
    let socket = AsyncTcpSocket::from(handle);
    let peer_addr = tcp_stream_peer_addr(handle)?;
    debug!(
        "tcp_stream_read on Thread {} from {}:{}",
        crate::libs::thread::current_thread_id(),
        peer_addr.0,
        peer_addr.1
    );
    block_on(socket.read(buffer), None)?.map_err(|_| ())
}

#[inline(always)]
pub fn tcp_stream_write(handle: Handle, buffer: &[u8]) -> Result<usize, ()> {
    let socket = AsyncTcpSocket::from(handle);
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
    block_on(socket.write(buffer), None)?.map_err(|err| {
        warn!("tcp_stream_write err {}", err);
        ()
    })
}

/// Close a TCP connection
#[inline(always)]
pub fn tcp_stream_close(handle: Handle) -> Result<(), ()> {
    let peer_addr = tcp_stream_peer_addr(handle)?;
    debug!(
        "tcp_stream_close T[{}] ip {}:{}",
        crate::libs::thread::current_thread_id(),
        peer_addr.0,
        peer_addr.1
    );
    let socket = AsyncTcpSocket::from(handle);
    block_on(socket.close(), None)?.map_err(|_| ())
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

#[inline(always)]
pub fn tcp_stream_shutdown(handle: Handle, how: Shutdown) -> Result<(), ()> {
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
pub fn tcp_stream_peer_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
    let mut guard = super::NIC.lock();
    let nic = guard.as_nic_mut().map_err(drop)?;
    let socket = nic.iface.get_socket::<TcpSocket<'_>>(handle);
    socket.set_keep_alive(Some(Duration::from_millis(DEFAULT_KEEP_ALIVE_INTERVAL)));
    let endpoint = socket.remote_endpoint();

    Ok((endpoint.addr, endpoint.port))
}

#[inline(always)]
pub fn tcp_stream_socket_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
    let mut guard = super::NIC.lock();
    let nic = guard.as_nic_mut().map_err(drop)?;
    let socket = nic.iface.get_socket::<TcpSocket<'_>>(handle);
    // socket.set_keep_alive(Some(Duration::from_millis(DEFAULT_KEEP_ALIVE_INTERVAL)));
    let endpoint = socket.local_endpoint();

    Ok((endpoint.addr, endpoint.port))
}

#[inline(always)]
pub fn tcp_listener_bind(ip: &[u8], port: u16) -> Result<u16, ()> {
    let ip = str::from_utf8(ip).map_err(|_| ())?;
    let port = if port == 0 {
        super::interface::get_local_endpoint()
    } else if !super::interface::check_local_endpoint(port) {
        port
    } else {
        warn!("tcp_listener_bind failed, port has been occupied");
        return Err(());
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
pub fn tcp_listener_accept(port: u16) -> Result<(Handle, IpAddress, u16), ()> {
    let local_endpoint = port;
    let socket = AsyncTcpSocket::new();
    let (addr, port) = block_on(socket.accept(port), None)?.map_err(|_| ())?;

    debug!(
        "tcp_listener_accept on Thread {} success on ip {} port {}, local_endpoint {}",
        crate::libs::thread::current_thread_id(),
        addr,
        port,
        local_endpoint
    );
    super::interface::set_local_endpoint_link(local_endpoint, IpEndpoint::new(addr, port));
    Ok((socket.inner(), addr, port))
}

/// If set, this option disables the Nagle algorithm. This means that segments are
/// always sent as soon as possible, even if there is only a small amount of data.
/// When not set, data is buffered until there is a sufficient amount to send out,
/// thereby avoiding the frequent sending of small packets.
#[inline(always)]
pub fn tcp_stream_set_no_delay(handle: Handle, mode: bool) -> Result<(), ()> {
    let mut guard = super::NIC.lock();
    let nic = guard.as_nic_mut().map_err(drop)?;
    let socket = nic.iface.get_socket::<TcpSocket<'_>>(handle);
    socket.set_nagle_enabled(!mode);

    Ok(())
}

#[inline(always)]
pub fn tcp_stream_no_delay(handle: Handle) -> Result<bool, ()> {
    let mut guard = super::NIC.lock();
    let nic = guard.as_nic_mut().map_err(drop)?;
    let socket = nic.iface.get_socket::<TcpSocket<'_>>(handle);
    Ok(socket.nagle_enabled())
}

#[inline(always)]
pub fn tcp_stream_set_nonblocking(_handle: Handle, mode: bool) -> Result<(), ()> {
    // non-blocking mode is currently not support
    // => return only an error, if `mode` is defined as `true`
    if mode {
        warn!("tcp_stream_set_nonblocking is not supported");
        Err(())
    } else {
        Ok(())
    }
}

#[inline(always)]
pub fn tcp_stream_set_read_timeout(_handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
    if timeout.is_none() {
        return Ok(());
    }
    warn!("tcp_stream_set_read_timeout is not supported");
    Err(())
}

#[inline(always)]
pub fn tcp_stream_get_read_timeout(_handle: Handle) -> Result<Option<u64>, ()> {
    warn!("tcp_stream_get_read_timeout is not supported");
    Ok(None)
}

#[inline(always)]
pub fn tcp_stream_set_write_timeout(_handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
    if timeout.is_none() {
        return Ok(());
    }
    warn!("tcp_stream_set_write_timeout is not supported");
    Err(())
}

#[inline(always)]
pub fn tcp_stream_get_write_timeout(_handle: Handle) -> Result<Option<u64>, ()> {
    warn!("tcp_stream_get_write_timeout is not supported");
    Ok(None)
}

#[inline(always)]
pub fn tcp_stream_duplicate(_handle: Handle) -> Result<Handle, ()> {
    warn!("tcp_stream_duplicate is not supported");
    Err(())
}

#[inline(always)]
pub fn tcp_stream_peek(_handle: Handle, _buf: &mut [u8]) -> Result<usize, ()> {
    warn!("tcp_stream_peek is not supported");
    Err(())
}

#[inline(always)]
pub fn tcp_stream_set_tll(_handle: Handle, _ttl: u32) -> Result<(), ()> {
    warn!("tcp_stream_set_tll is not supported");
    Err(())
}

#[inline(always)]
pub fn tcp_stream_get_tll(_handle: Handle) -> Result<u32, ()> {
    warn!("tcp_stream_get_tll is not supported");
    Err(())
}
