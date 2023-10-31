use alloc::boxed::Box;
use alloc::vec;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU16, Ordering};
use core::ops::DerefMut;

use smoltcp::iface::SocketSet;
use smoltcp::socket::{tcp, udp, AnySocket};
use smoltcp::time::{Duration, Instant};

use crate::libs::error::ShyperError;
use crate::libs::synch::spinlock::SpinlockIrqSave;
use crate::libs::net::SmoltcpSocketHandle;
use crate::libs::net::device::ShyperNet;

const TCP_RX_BUF_LEN: usize = 64 * 1024;
const TCP_TX_BUF_LEN: usize = 64 * 1024;
const UDP_RX_BUF_LEN: usize = 64 * 1024;
const UDP_TX_BUF_LEN: usize = 64 * 1024;

#[inline]
pub(crate) fn now() -> Instant {
    Instant::from_millis(crate::libs::timer::current_ms() as i64)
}

pub enum NetworkState<'a> {
    Missing,
    InitializationFailed,
    Initialized(Box<NetworkInterface<'a>>),
}

impl<'a> NetworkState<'a> {
    pub fn as_nic_mut(&mut self) -> Result<&mut NetworkInterface<'a>, ShyperError> {
        match self {
            NetworkState::Initialized(nic) => Ok(nic),
            _ => {
                warn!("Network is not initialized!");
                Err(ShyperError::BadState)
            }
        }
    }
}

pub(crate) static NIC: SpinlockIrqSave<NetworkState> = SpinlockIrqSave::new(NetworkState::Missing);

pub struct NetworkInterface<'a> {
    pub(super) iface: smoltcp::iface::Interface,
    pub(super) sockets: SocketSet<'a>,
    pub(super) device: ShyperNet,
}

impl<'a> NetworkInterface<'a> {
    pub fn create_tcp_handle(&mut self) -> Result<SmoltcpSocketHandle, ()> {
        let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]);
        let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]);
        let mut tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);
        tcp_socket.set_nagle_enabled(true);
        let tcp_handle = self.sockets.add(tcp_socket);

        Ok(tcp_handle)
    }

    pub fn create_udp_handle(&mut self) -> Result<SmoltcpSocketHandle, ()> {
        // Must fit mDNS payload of at least one packet
        let udp_rx_buffer =
            udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 8], vec![0; UDP_RX_BUF_LEN]);
        // Will not send mDNS
        let udp_tx_buffer =
            udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 8], vec![0; UDP_TX_BUF_LEN]);
        let udp_socket = udp::Socket::new(udp_rx_buffer, udp_tx_buffer);
        let udp_handle = self.sockets.add(udp_socket);

        Ok(udp_handle)
    }

    pub fn poll_common(&mut self, timestamp: Instant) {
        let _readiness_may_have_changed =
            self.iface
                .poll(timestamp, &mut self.device, &mut self.sockets);
        // debug!("poll common at {timestamp} {_readiness_may_have_changed}");
    }

    pub(crate) fn poll_delay(&mut self, timestamp: Instant) -> Option<Duration> {
        self.iface.poll_delay(timestamp, &self.sockets)
    }

    #[allow(dead_code)]
    pub(crate) fn get_socket<T: AnySocket<'a>>(&self, handle: SmoltcpSocketHandle) -> &T {
        self.sockets.get(handle)
    }

    pub(crate) fn get_mut_socket<T: AnySocket<'a>>(
        &mut self,
        handle: SmoltcpSocketHandle,
    ) -> &mut T {
        self.sockets.get_mut(handle)
    }

    pub(crate) fn get_socket_and_context<T: AnySocket<'a>>(
        &mut self,
        handle: SmoltcpSocketHandle,
    ) -> (&mut T, &mut smoltcp::iface::Context) {
        (self.sockets.get_mut(handle), self.iface.context())
    }
}

static LOCAL_ENDPOINT: AtomicU16 = AtomicU16::new(0);

use smoltcp::wire::IpEndpoint;

static LOCAL_ENDPOINT_MAP: SpinlockIrqSave<BTreeMap<u16, Option<IpEndpoint>>> =
    SpinlockIrqSave::new(BTreeMap::new());

pub fn network_init() {
    info!("network_init() lib init");
    // Initialize variable, which contains the next local endpoint
    LOCAL_ENDPOINT.store(start_endpoint(), Ordering::Relaxed);

    let mut guard = NIC.lock();

    *guard = NetworkInterface::new();

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
            crate::libs::thread::thread_block_current_with_timeout(delay_millis as usize);
        }

		#[cfg(feature = "async-net")]
        super::executor::spawn(network_run()).detach();
    } else {
        warn!("network_init, NetworkState is not Initialized!");
    }
    info!("network_init() lib init finished");
}

#[cfg(feature = "async-net")]
async fn network_run() {
	use core::task::Poll;
	
    debug!("network_run");
    futures_lite::future::poll_fn(|cx| match NIC.lock().deref_mut() {
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

#[inline]
pub fn network_poll() {
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

// Todo: may failed.
pub fn get_ephemeral_port() -> Result<u16, ShyperError> {
    let mut lock = LOCAL_ENDPOINT_MAP.lock();
    let mut local_endpoint;
    loop {
        local_endpoint = LOCAL_ENDPOINT.fetch_add(1, Ordering::SeqCst);
        if lock.contains_key(&local_endpoint) {
            continue;
        }
        if local_endpoint > u16::MAX {
            warn!("get_ephemeral_port failed, port exceeds u16 max");
            // Let's just start over.
            LOCAL_ENDPOINT.store(start_endpoint(), Ordering::Relaxed);
        }
        lock.insert(local_endpoint, None);
        break;
    }
    Ok(local_endpoint)
}

pub fn check_local_endpoint(endpoint: u16) -> bool {
    let lock = LOCAL_ENDPOINT_MAP.lock();
    lock.contains_key(&endpoint)
}
