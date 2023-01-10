use alloc::str::FromStr;
use alloc::{str, vec};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicU16, Ordering};
use core::task::Poll;

use futures_lite::future;
use smoltcp::phy::Device;
use smoltcp::iface::{self, SocketHandle};
use smoltcp::socket::{TcpSocket, TcpSocketBuffer, TcpState};
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::IpAddress;

use smoltcp::Error;

use crate::libs::timer::current_ms;
use crate::libs::synch::spinlock::SpinlockIrqSave;
use crate::thread_block_current_with_timeout;

use super::device::ShyperNet;
use super::executor::{block_on, spawn};

pub enum NetworkState {
    Missing,
    InitializationFailed,
    Initialized(Box<NetworkInterface<ShyperNet>>),
}

impl NetworkState {
    pub fn as_nic_mut(&mut self) -> Result<&mut NetworkInterface<ShyperNet>, &'static str> {
        match self {
            NetworkState::Initialized(nic) => Ok(nic),
            _ => Err("Network is not initialized!"),
        }
    }
}

pub type Handle = SocketHandle;

/// Default keep alive interval in milliseconds
const DEFAULT_KEEP_ALIVE_INTERVAL: u64 = 75000;

static LOCAL_ENDPOINT: AtomicU16 = AtomicU16::new(0);

use smoltcp::wire::IpEndpoint;

static LOCAL_ENDPOINT_MAP: SpinlockIrqSave<BTreeMap<u16, Option<IpEndpoint>>> =
    SpinlockIrqSave::new(BTreeMap::new());

fn start_endpoint() -> u16 {
    // use cortex_a::registers::CNTPCT_EL0;
    // use tock_registers::interfaces::Readable;
    // let start_endpoint: u16= (CNTPCT_EL0.get() % (u16::MAX as u64)).try_into().unwrap();
    // if start_endpoint < 1024 {
    //     start_endpoint += 1024;
    // }
    let start_endpoint = 4444;
    debug!("get start endpoint {}", start_endpoint);
    start_endpoint
}

fn get_local_endpoint() -> u16 {
    let mut lock = LOCAL_ENDPOINT_MAP.lock();
    let mut local_endpoint;
    loop {
        local_endpoint = LOCAL_ENDPOINT.fetch_add(1, Ordering::SeqCst);
        if lock.contains_key(&local_endpoint) {
            continue;
        }
        if local_endpoint > u16::MAX {
            warn!("get_local_endpoint failed, port exceeds u16 max");
            // Let's just start over.
            LOCAL_ENDPOINT.store(start_endpoint(), Ordering::Relaxed);
        }
        lock.insert(local_endpoint, None);
        break;
    }
    local_endpoint
}

fn check_local_endpoint(endpoint: u16) -> bool {
    let lock = LOCAL_ENDPOINT_MAP.lock();
    lock.contains_key(&endpoint)
}

fn set_local_endpoint_link(local_endpoint: u16, remote_endpoint: IpEndpoint) {
    let mut lock = LOCAL_ENDPOINT_MAP.lock();
    if !lock.contains_key(&local_endpoint) {
        warn!("local endpoint not exists");
        return;
    }
    if lock.get(&local_endpoint).unwrap().is_some() {
        warn!("local endpoint has been occupied");
    }
    lock.insert(local_endpoint, Some(remote_endpoint));
}

fn remove_local_endpoint(local_endpoint: u16) {
    let mut lock = LOCAL_ENDPOINT_MAP.lock();
    let remote_endpoint = lock.remove(&local_endpoint).unwrap();
    if remote_endpoint.is_some() {
        debug!(
            "connect to remote {} is closed, release port {}",
            remote_endpoint.unwrap(),
            local_endpoint
        );
    }
}

pub(crate) static NIC: SpinlockIrqSave<NetworkState> = SpinlockIrqSave::new(NetworkState::Missing);

pub struct NetworkInterface<T: for<'a> Device<'a>> {
    pub iface: smoltcp::iface::Interface<'static, T>,
}

impl<T> NetworkInterface<T>
where
    T: for<'a> Device<'a>,
{
    pub fn create_handle(&mut self) -> Result<Handle, ()> {
        let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 65535]);
        let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 65535]);
        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
        let tcp_handle = self.iface.add_socket(tcp_socket);

        Ok(tcp_handle)
    }

    pub fn poll_common(&mut self, timestamp: Instant) {
        // let mut start = crate::libs::timer::current_us();
        // let _start = start;
        while self.iface.poll(timestamp).unwrap_or(true) {
            // just to make progress
            // debug!("NetworkInterface::poll_common::poll:send or receive packets!!!");
            // let end = crate::libs::timer::current_us();
            // println!("poll_common , one pull use {} us, current {} us", end - start, end);
            // start = crate::libs::timer::current_us();
        }
        // println!("poll_common end , totally use {} us, current {} us", start - _start, start);
    }

    pub fn poll_delay(&mut self, timestamp: Instant) -> Option<Duration> {
        self.iface.poll_delay(timestamp)
    }
}

pub struct AsyncSocket(Handle);

impl AsyncSocket {
    pub fn new() -> Self {
        let handle = NIC.lock().as_nic_mut().unwrap().create_handle().unwrap();
        // println!("create handle {:?}",handle);
        Self(handle)
    }

    pub(crate) fn inner(&self) -> Handle {
        self.0
    }

    fn with<R>(&self, f: impl FnOnce(&mut TcpSocket<'_>) -> R) -> R {
        // println!("Async Socket with(), handle {:?}", self.0);

        let mut guard = NIC.lock();
        let nic = guard.as_nic_mut().unwrap();
        let res = {
            // let start = crate::libs::timer::current_us();

            let s = nic.iface.get_socket::<TcpSocket<'_>>(self.0);
            let res = f(s);

            // let end = crate::libs::timer::current_us();
            // println!(
            //     "AsyncSocket with() f use {} us, current {} us",
            //     end - start,
            //     end
            // );
            res
        };
        // To flush send buffers.
        // After using the socket, the network interface has to poll the nic,
        // This is required to flush all send buffers.
        let t = now();
        // let start = t.total_micros() as usize;
        if nic.poll_delay(t).map(|d| d.total_millis()).unwrap_or(0) == 0 {
            nic.poll_common(t);
        }
        // let end = crate::libs::timer::current_us();
        // println!(
        //     "AsyncSocket with() poll_common use {} us, current {} us",
        //     end - start,
        //     end
        // );
        res
    }

    fn with_context<R>(
        &self,
        f: impl FnOnce(&mut TcpSocket<'_>, &mut iface::Context<'_>) -> R,
    ) -> R {
        let mut guard = NIC.lock();
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
        trace!("AsyncSocket accept");
        self.with(|socket| socket.listen(port).map_err(|_| Error::Illegal))?;

        future::poll_fn(|cx| {
            self.with(|socket| {
                if socket.is_active() {
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

        let mut guard = NIC.lock();
        let nic = guard.as_nic_mut().map_err(|_| Error::Illegal)?;
        let socket = nic.iface.get_socket::<TcpSocket<'_>>(self.0);
        socket.set_keep_alive(Some(Duration::from_millis(DEFAULT_KEEP_ALIVE_INTERVAL)));
        let endpoint = socket.remote_endpoint();

        Ok((endpoint.addr, endpoint.port))
    }

    pub async fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        future::poll_fn(|cx| {
            self.with(|socket| match socket.state() {
                TcpState::FinWait1
                | TcpState::FinWait2
                | TcpState::Closed
                | TcpState::Closing
                | TcpState::TimeWait => Poll::Ready(Err(Error::Illegal)),
                _ => {
                    if socket.may_recv() {
                        let n = socket.recv_slice(buffer).map_err(|_| Error::Illegal)?;
                        if n > 0 || buffer.is_empty() {
                            return Poll::Ready(Ok(n));
                        }
                    }

                    socket.register_recv_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await
    }

    pub async fn write(&self, buffer: &[u8]) -> Result<usize, Error> {
        let len = buffer.len();
        let mut pos: usize = 0;

        while pos < len {
            let n = future::poll_fn(|cx| {
                self.with(|socket| match socket.state() {
                    TcpState::FinWait1
                    | TcpState::FinWait2
                    | TcpState::Closed
                    | TcpState::Closing
                    | TcpState::TimeWait => Poll::Ready(Err(Error::Illegal)),
                    _ => {
                        if !socket.may_send() {
                            return Poll::Ready(Err(Error::Illegal));
                        } else if socket.can_send() {
                            return Poll::Ready(
                                socket
                                    .send_slice(&buffer[pos..])
                                    .map_err(|_| Error::Illegal),
                            );
                        }

                        if pos > 0 {
                            // we already send some data => return 0 as signal to stop the
                            // async write
                            return Poll::Ready(Ok(0));
                        }

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
                        remove_local_endpoint(socket.local_endpoint().port);
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

impl From<Handle> for AsyncSocket {
    fn from(handle: Handle) -> Self {
        AsyncSocket(handle)
    }
}

#[inline]
pub(crate) fn now() -> Instant {
    Instant::from_millis(current_ms() as i64)
}

async fn network_run() {
    debug!("network_run");
    future::poll_fn(|cx| match NIC.lock().deref_mut() {
        NetworkState::Initialized(nic) => {
            nic.poll_common(now());

            // this background task will never stop
            // => wakeup ourself
            cx.waker().clone().wake();

            Poll::Pending
        }
        _ => Poll::Ready(()),
    })
    .await
}

// extern "C" fn nic_thread(_: usize) {
//     info!("[nic_thread] Enter NIC thread\n*********************************************\n");
//     loop {
//         debug!("[nic_thread] enter netwait");

//         netwait();

//         debug!("[nic_thread] netwait finished, try to call nic.poll_common");

//         if let NetworkState::Initialized(nic) = NIC.lock().deref_mut() {
//             // debug!("NetworkState Initialized success, poll_common");
//             nic.poll_common(Instant::from_millis(current_ms() as i64));
//             nic.wake();
//         }
//     }
// }

#[inline]
pub fn network_poll() {
    // debug!("network poll");
    if let Ok(mut guard) = NIC.try_lock() {
        if let NetworkState::Initialized(nic) = guard.deref_mut() {
            let time = now();
            nic.poll_common(time);
            // if let Some(delay) = nic.poll_delay(time).map(|d| d.total_micros()) {
            //     debug!("network poll, get delay {}", delay);
            // }
        }
    }
}

pub fn network_init() {
    info!("network_init() lib init");
    // Initialize variable, which contains the next local endpoint
    LOCAL_ENDPOINT.store(start_endpoint(), Ordering::Relaxed);

    let mut guard = NIC.lock();

    *guard = NetworkInterface::<ShyperNet>::new();

    if let NetworkState::Initialized(nic) = guard.deref_mut() {
        let time = now();
        nic.poll_common(now());

        if let Some(delay_millis) = nic.poll_delay(time).map(|d| d.total_millis()) {
            // let wakeup_time = crate::arch::processor::get_timer_ticks() + delay;
            // crate::core_scheduler().add_network_timer(wakeup_time);
            info!(
                "network_init() initialized get poll delay {} ms, now {} ms",
                delay_millis,
                time.millis()
            );
            thread_block_current_with_timeout(delay_millis as usize);
        }

        spawn(network_run()).detach();
    }
    info!("network_init() lib init finished");
}

#[inline(always)]
pub fn tcp_stream_connect(ip: &[u8], port: u16, timeout: Option<u64>) -> Result<Handle, ()> {
    let socket = AsyncSocket::new();
    let local_endpoint = get_local_endpoint();
    let address = IpAddress::from_str(str::from_utf8(ip).map_err(|_| ())?).map_err(|_| ())?;
    let res = block_on(
        socket.connect(address, port, local_endpoint),
        timeout.map(Duration::from_millis),
    )?
    .map_err(|_| ());
    set_local_endpoint_link(local_endpoint, IpEndpoint::new(address, port));
    res
}

#[inline(always)]
pub fn tcp_stream_read(handle: Handle, buffer: &mut [u8]) -> Result<usize, ()> {
    let socket = AsyncSocket::from(handle);
    block_on(socket.read(buffer), None)?.map_err(|_| ())
}

#[inline(always)]
pub fn tcp_stream_write(handle: Handle, buffer: &[u8]) -> Result<usize, ()> {
    let socket = AsyncSocket::from(handle);
    block_on(socket.write(buffer), None)?.map_err(|_| ())
}

#[inline(always)]
pub fn tcp_stream_close(handle: Handle) -> Result<(), ()> {
    let socket = AsyncSocket::from(handle);
    block_on(socket.close(), None)?.map_err(|_| ())
}

//ToDo: an enum, or at least constants would be better
#[inline(always)]
pub fn tcp_stream_shutdown(handle: Handle, how: i32) -> Result<(), ()> {
    match how {
		0 /* Read */ => {
			trace!("Shutdown::Read is not implemented");
			Ok(())
		},
		1 /* Write */ => {
			tcp_stream_close(handle)
		},
		2 /* Both */ => {
			tcp_stream_close(handle)
		},
		_ => {
			panic!("Invalid shutdown argument {}", how);
		},
	}
}

#[inline(always)]
pub fn tcp_stream_peer_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
    let mut guard = NIC.lock();
    let nic = guard.as_nic_mut().map_err(drop)?;
    let socket = nic.iface.get_socket::<TcpSocket<'_>>(handle);
    socket.set_keep_alive(Some(Duration::from_millis(DEFAULT_KEEP_ALIVE_INTERVAL)));
    let endpoint = socket.remote_endpoint();

    Ok((endpoint.addr, endpoint.port))
}

#[inline(always)]
pub fn tcp_listener_bind(ip: &[u8], port: u16) -> Result<u16, ()> {
    let ip = str::from_utf8(ip).map_err(|_| ())?;
    let port = if port == 0 {
        get_local_endpoint()
    } else if !check_local_endpoint(port) {
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

#[inline(always)]
pub fn tcp_listener_accept(port: u16) -> Result<(Handle, IpAddress, u16), ()> {
    let local_endpoint = port;
    debug!("tcp_listener_accept on local endpoint {}", local_endpoint);
    let socket = AsyncSocket::new();
    let (addr, port) = block_on(socket.accept(port), None)?.map_err(|_| ())?;

    set_local_endpoint_link(local_endpoint, IpEndpoint::new(addr, port));
    debug!(
        "tcp_listener_accept success on ip {} port {}, local_endpoint {}",
        addr, port, local_endpoint
    );
    Ok((socket.inner(), addr, port))
}

/// If set, this option disables the Nagle algorithm. This means that segments are
/// always sent as soon as possible, even if there is only a small amount of data.
/// When not set, data is buffered until there is a sufficient amount to send out,
/// thereby avoiding the frequent sending of small packets.
#[inline(always)]
pub fn tcp_set_no_delay(handle: Handle, mode: bool) -> Result<(), ()> {
    let mut guard = NIC.lock();
    let nic = guard.as_nic_mut().map_err(drop)?;
    let socket = nic.iface.get_socket::<TcpSocket<'_>>(handle);
    socket.set_nagle_enabled(!mode);

    Ok(())
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

#[deprecated(since = "0.1.14", note = "Please don't use this function")]
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
