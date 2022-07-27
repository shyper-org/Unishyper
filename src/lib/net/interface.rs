use alloc::str::FromStr;
use alloc::{str, vec};
use core::ops::DerefMut;
use core::sync::atomic::{AtomicU16, Ordering};
use core::task::{Context, Poll};
use spin::Mutex;

use futures_lite::future;
use lazy_static::lazy_static;
#[cfg(feature = "dhcpv4")]
use smoltcp::dhcp::Dhcpv4Client;
use smoltcp::phy::Device;

use smoltcp::socket::{SocketHandle, SocketSet, TcpSocket, TcpSocketBuffer, TcpState};
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::IpAddress;

#[cfg(feature = "dhcpv4")]
use smoltcp::wire::{IpCidr, Ipv4Address, Ipv4Cidr};
use smoltcp::Error;

use crate::drivers::net::netwait;
use crate::exported::thread_spawn;
use crate::lib::thread::thread_yield;
use crate::lib::timer::current_ms;

use super::device::ShyperNet;
use super::executor::{block_on, spawn};
use super::waker::WakerRegistration;

pub enum NetworkState {
    Missing,
    InitializationFailed,
    Initialized(NetworkInterface<ShyperNet>),
}

impl NetworkState {
    fn as_nic_mut(&mut self) -> Result<&mut NetworkInterface<ShyperNet>, &'static str> {
        match self {
            NetworkState::Initialized(nic) => Ok(nic),
            _ => Err("Network is not initialized!"),
        }
    }
}

lazy_static! {
    static ref NIC: Mutex<NetworkState> = Mutex::new(NetworkState::Missing);
}

type Handle = SocketHandle;

/// Default keep alive interval in milliseconds
const DEFAULT_KEEP_ALIVE_INTERVAL: u64 = 75000;

static LOCAL_ENDPOINT: AtomicU16 = AtomicU16::new(0);

pub struct NetworkInterface<T: for<'a> Device<'a>> {
    pub iface: smoltcp::iface::EthernetInterface<'static, T>,
    pub sockets: SocketSet<'static>,
    #[cfg(feature = "dhcpv4")]
    dhcp: Dhcpv4Client,
    #[cfg(feature = "dhcpv4")]
    prev_cidr: Ipv4Cidr,
    pub waker: WakerRegistration,
}

impl<T> NetworkInterface<T>
where
    T: for<'a> Device<'a>,
{
    pub fn create_handle(&mut self) -> Result<Handle, ()> {
        let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 65535]);
        let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 65535]);
        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
        let tcp_handle = self.sockets.add(tcp_socket);

        Ok(tcp_handle)
    }

    pub fn wake(&mut self) {
        self.waker.wake()
    }

    pub fn poll_common(&mut self, timestamp: Instant) {
        while self
            .iface
            .poll(&mut self.sockets, timestamp)
            .unwrap_or(true)
        {
            // just to make progress
            // debug!("NetworkInterface::poll_common::poll:send or receive packets!!!");
        }
        #[cfg(feature = "dhcpv4")]
        let config = self
            .dhcp
            .poll(&mut self.iface, &mut self.sockets, timestamp)
            .unwrap_or_else(|e| {
                debug!("DHCP: {:?}", e);
                None
            });
        #[cfg(feature = "dhcpv4")]
        config.map(|config| {
            debug!("DHCP config: {:?}", config);
            if let Some(cidr) = config.address {
                if cidr != self.prev_cidr && !cidr.address().is_unspecified() {
                    self.iface.update_ip_addrs(|addrs| {
                        addrs.iter_mut().next().map(|addr| {
                            *addr = IpCidr::Ipv4(cidr);
                        });
                    });
                    self.prev_cidr = cidr;
                    debug!("Assigned a new IPv4 address: {}", cidr);
                }
            }

            config.router.map(|router| {
                self.iface
                    .routes_mut()
                    .add_default_ipv4_route(router)
                    .unwrap()
            });
            self.iface.routes_mut().update(|routes_map| {
                routes_map
                    .get(&IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0))
                    .map(|default_route| {
                        debug!("Default gateway: {}", default_route.via_router);
                    });
            });

            if config.dns_servers.iter().any(|s| s.is_some()) {
                debug!("DNS servers:");
                for dns_server in config.dns_servers.iter().filter_map(|s| *s) {
                    debug!("- {}", dns_server);
                }
            }
        });
    }

    pub fn poll(&mut self, cx: &mut Context<'_>, timestamp: Instant) {
        self.waker.register(cx.waker());
        self.poll_common(timestamp);
    }

    pub fn poll_delay(&mut self, timestamp: Instant) -> Option<Duration> {
        self.iface.poll_delay(&self.sockets, timestamp)
    }
}

pub struct AsyncSocket(Handle);

impl AsyncSocket {
    pub fn new() -> Self {
        let handle = NIC.lock().as_nic_mut().unwrap().create_handle().unwrap();
        // println!("create handle {:?}",handle);
        Self(handle)
    }

    fn with<R>(&self, f: impl FnOnce(&mut TcpSocket) -> R) -> R {
        // println!("Async Socket with(), handle {:?}", self.0);

        let mut guard = NIC.lock();
        let nic = guard.as_nic_mut().unwrap();
        let res = {
            let mut s = nic.sockets.get::<TcpSocket>(self.0);
            f(&mut *s)
        };
        nic.wake();
        // To flush send buffers.
        // After using the socket, the network interface has to poll the nic,
        // This is required to flush all send buffers.
		let _ = nic.iface.poll(&mut nic.sockets,Instant::from_millis(current_ms() as i64));
        res
    }

    pub async fn connect(&self, ip: &[u8], port: u16) -> Result<Handle, Error> {
        let address = IpAddress::from_str(str::from_utf8(ip).map_err(|_| Error::Illegal)?)
            .map_err(|_| Error::Illegal)?;

        self.with(|s| {
            s.connect(
                (address, port),
                LOCAL_ENDPOINT.fetch_add(1, Ordering::SeqCst),
            )
        })
        .map_err(|_| Error::Illegal)?;

        future::poll_fn(|cx| {
            self.with(|s| match s.state() {
                TcpState::Closed | TcpState::TimeWait => Poll::Ready(Err(Error::Unaddressable)),
                TcpState::Listen => Poll::Ready(Err(Error::Illegal)),
                TcpState::SynSent | TcpState::SynReceived => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(self.0)),
            })
        })
        .await
    }

    pub async fn accept(&self, port: u16) -> Result<(IpAddress, u16), Error> {
        trace!("AsyncSocket accept");
        self.with(|s| s.listen(port).map_err(|_| Error::Illegal))?;

        future::poll_fn(|cx| {
            self.with(|s| {
                if s.is_active() {
                    Poll::Ready(Ok(()))
                } else {
                    match s.state() {
                        TcpState::Closed
                        | TcpState::Closing
                        | TcpState::FinWait1
                        | TcpState::FinWait2 => Poll::Ready(Err(Error::Illegal)),
                        _ => {
                            s.register_recv_waker(cx.waker());
                            Poll::Pending
                        }
                    }
                }
            })
        })
        .await?;

        let mut guard = NIC.lock();
        let nic = guard.as_nic_mut().map_err(|_| Error::Illegal)?;
        let mut socket = nic.sockets.get::<TcpSocket>(self.0);
        socket.set_keep_alive(Some(Duration::from_millis(DEFAULT_KEEP_ALIVE_INTERVAL)));
        let endpoint = socket.remote_endpoint();

        Ok((endpoint.addr, endpoint.port))
    }

    pub async fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        future::poll_fn(|cx| {
            self.with(|s| match s.state() {
                TcpState::FinWait1
                | TcpState::FinWait2
                | TcpState::Closed
                | TcpState::Closing
                | TcpState::TimeWait => Poll::Ready(Err(Error::Illegal)),
                _ => {
                    if s.may_recv() {
                        let n = s.recv_slice(buffer)?;
                        if n > 0 || buffer.is_empty() {
                            return Poll::Ready(Ok(n));
                        }
                    }

                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await
        .map_err(|_| Error::Illegal)
    }

    pub async fn write(&self, buffer: &[u8]) -> Result<usize, Error> {
        future::poll_fn(|cx| {
            self.with(|s| match s.state() {
                TcpState::FinWait1
                | TcpState::FinWait2
                | TcpState::Closed
                | TcpState::Closing
                | TcpState::TimeWait => Poll::Ready(Err(Error::Illegal)),
                _ => {
                    if !s.may_recv() {
                        Poll::Ready(Ok(0))
                    } else if s.can_send() {
                        Poll::Ready(s.send_slice(buffer).map_err(|_| Error::Illegal))
                    } else {
                        s.register_send_waker(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
        .await
    }

    pub async fn close(&self) -> Result<(), Error> {
        future::poll_fn(|cx| {
            self.with(|s| match s.state() {
                TcpState::FinWait1
                | TcpState::FinWait2
                | TcpState::Closed
                | TcpState::Closing
                | TcpState::TimeWait => Poll::Ready(Err(Error::Illegal)),
                _ => {
                    if s.send_queue() > 0 {
                        s.register_send_waker(cx.waker());
                        Poll::Pending
                    } else {
                        s.close();
                        Poll::Ready(Ok(()))
                    }
                }
            })
        })
        .await?;

        future::poll_fn(|cx| {
            self.with(|s| match s.state() {
                TcpState::FinWait1
                | TcpState::FinWait2
                | TcpState::Closed
                | TcpState::Closing
                | TcpState::TimeWait => Poll::Ready(Ok(())),
                _ => {
                    s.register_send_waker(cx.waker());
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

fn start_endpoint() -> u16 {
    use cortex_a::registers::CNTPCT_EL0;
    use tock_registers::interfaces::Readable;
    debug!("get start endpoint {}", CNTPCT_EL0.get());
    (CNTPCT_EL0.get() % (u16::MAX as u64)).try_into().unwrap()
}

pub fn network_delay(timestamp: Instant) -> Option<Duration> {
    NIC.lock().as_nic_mut().ok()?.poll_delay(timestamp)
}

pub async fn network_run() {
    debug!("network_run");
    future::poll_fn(|cx| match NIC.lock().deref_mut() {
        NetworkState::Initialized(nic) => {
            nic.poll(cx, Instant::from_millis(current_ms() as i64));
            Poll::Pending
        }
        _ => Poll::Ready(()),
    })
    .await
}

extern "C" fn nic_thread(_: usize) {
    info!("[nic_thread] Enter NIC thread\n*********************************************\n");
    loop {
        debug!("[nic_thread] enter netwait");

        netwait();

        debug!("[nic_thread] netwait finished, try to call nic.poll_common");

        if let NetworkState::Initialized(nic) = NIC.lock().deref_mut() {
            // debug!("NetworkState Initialized success, poll_common");
            nic.poll_common(Instant::from_millis(current_ms() as i64));
            nic.wake();
        }
    }
}

pub fn network_init() {
    info!("network lib init");
    // Initialize variable, which contains the next local endpoint
    LOCAL_ENDPOINT.store(start_endpoint(), Ordering::SeqCst);

    let mut guard = NIC.lock();

    *guard = NetworkInterface::<ShyperNet>::new();

    if let NetworkState::Initialized(nic) = guard.deref_mut() {
        nic.poll_common(Instant::from_millis(current_ms() as i64));

        // Create thread, which manages the network stack
        // Todo: may use a higher priority to reduce the network latency
        let tid = thread_spawn(nic_thread, 0);
        info!("Spawn network thread with id {}", tid);

        spawn(network_run()).detach();

        // switch to network thread
        thread_yield();
    }
    info!("network lib init finished");
}

#[no_mangle]
pub fn tcp_stream_connect(ip: &[u8], port: u16, timeout: Option<u64>) -> Result<Handle, ()> {
    let socket = AsyncSocket::new();
    block_on(socket.connect(ip, port), timeout.map(Duration::from_millis))?.map_err(|_| ())
}

#[no_mangle]
pub fn tcp_stream_read(handle: Handle, buffer: &mut [u8]) -> Result<usize, ()> {
    let socket = AsyncSocket::from(handle);
    block_on(socket.read(buffer), None)?.map_err(|_| ())
}

#[no_mangle]
pub fn tcp_stream_write(handle: Handle, buffer: &[u8]) -> Result<usize, ()> {
    let socket = AsyncSocket::from(handle);
    block_on(socket.write(buffer), None)?.map_err(|_| ())
}

#[no_mangle]
pub fn tcp_stream_close(handle: Handle) -> Result<(), ()> {
    let socket = AsyncSocket::from(handle);
    block_on(socket.close(), None)?.map_err(|_| ())
}

//ToDo: an enum, or at least constants would be better
#[no_mangle]
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

#[no_mangle]
pub fn tcp_stream_set_read_timeout(_handle: Handle, _timeout: Option<u64>) -> Result<(), ()> {
    Err(())
}

#[no_mangle]
pub fn tcp_stream_get_read_timeout(_handle: Handle) -> Result<Option<u64>, ()> {
    Err(())
}

#[no_mangle]
pub fn tcp_stream_set_write_timeout(_handle: Handle, _timeout: Option<u64>) -> Result<(), ()> {
    Err(())
}

#[no_mangle]
pub fn tcp_stream_get_write_timeout(_handle: Handle) -> Result<Option<u64>, ()> {
    Err(())
}

#[deprecated(since = "0.1.14", note = "Please don't use this function")]
#[no_mangle]
pub fn tcp_stream_duplicate(_handle: Handle) -> Result<Handle, ()> {
    Err(())
}

#[no_mangle]
pub fn tcp_stream_peek(_handle: Handle, _buf: &mut [u8]) -> Result<usize, ()> {
    Err(())
}

#[no_mangle]
pub fn tcp_stream_set_nonblocking(_handle: Handle, mode: bool) -> Result<(), ()> {
    // non-blocking mode is currently not support
    // => return only an error, if `mode` is defined as `true`
    if mode {
        Err(())
    } else {
        Ok(())
    }
}

#[no_mangle]
pub fn tcp_stream_set_tll(_handle: Handle, _ttl: u32) -> Result<(), ()> {
    Err(())
}

#[no_mangle]
pub fn tcp_stream_get_tll(_handle: Handle) -> Result<u32, ()> {
    Err(())
}

#[no_mangle]
pub fn tcp_stream_peer_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
    let mut guard = NIC.lock();
    let nic = guard.as_nic_mut().map_err(drop)?;
    let mut socket = nic.sockets.get::<TcpSocket>(handle);
    socket.set_keep_alive(Some(Duration::from_millis(DEFAULT_KEEP_ALIVE_INTERVAL)));
    let endpoint = socket.remote_endpoint();

    Ok((endpoint.addr, endpoint.port))
}

#[no_mangle]
pub fn tcp_listener_accept(port: u16) -> Result<(Handle, IpAddress, u16), ()> {
    debug!("tcp_listener_accept");
    let socket = AsyncSocket::new();
    let (addr, port) = block_on(socket.accept(port), None)?.map_err(|_| ())?;

    Ok((socket.0, addr, port))
}
