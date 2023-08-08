use alloc::boxed::Box;
use alloc::vec;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU16, Ordering};
use core::task::Poll;
use core::ops::DerefMut;

use futures_lite::future;
use smoltcp::phy::Device;
use smoltcp::socket::{TcpSocket, TcpSocketBuffer};
use smoltcp::socket::{UdpSocket, UdpSocketBuffer, UdpPacketMetadata};
use smoltcp::time::{Duration, Instant};

use crate::libs::synch::spinlock::SpinlockIrqSave;
use crate::libs::net::Handle;
use crate::libs::net::device::ShyperNet;

#[inline]
pub(crate) fn now() -> Instant {
    Instant::from_millis(crate::libs::timer::current_ms() as i64)
}


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

pub(crate) static NIC: SpinlockIrqSave<NetworkState> = SpinlockIrqSave::new(NetworkState::Missing);

pub struct NetworkInterface<T: for<'a> Device<'a>> {
    pub iface: smoltcp::iface::Interface<'static, T>,
}

impl<T> NetworkInterface<T>
where
    T: for<'a> Device<'a>,
{
    pub fn create_tcp_handle(&mut self) -> Result<Handle, ()> {
        let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 65535]);
        let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 65535]);
        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
        let tcp_handle = self.iface.add_socket(tcp_socket);

        Ok(tcp_handle)
    }

    pub fn create_udp_handle(&mut self) -> Result<Handle, ()> {
        let udp_rx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY; 8], vec![0; 65535]);
        let udp_tx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY; 8], vec![0; 65535]);
        let ucp_socket = UdpSocket::new(udp_rx_buffer, udp_tx_buffer);
        let udp_handle = self.iface.add_socket(ucp_socket);

        Ok(udp_handle)
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
            core::hint::spin_loop();
        }
        // println!("poll_common end , totally use {} us, current {} us", start - _start, start);
    }

    pub fn poll_delay(&mut self, timestamp: Instant) -> Option<Duration> {
        self.iface.poll_delay(timestamp)
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
            crate::libs::thread::thread_block_current_with_timeout(delay_millis as usize);
        }

        super::executor::spawn(network_run()).detach();
    } else {
        warn!("network_init, NetworkState is not Initialized!");
    }
    info!("network_init() lib init finished");
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

pub fn get_local_endpoint() -> u16 {
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

pub fn check_local_endpoint(endpoint: u16) -> bool {
    let lock = LOCAL_ENDPOINT_MAP.lock();
    lock.contains_key(&endpoint)
}

// Todo: can one port be connected to multiple remote endpoint?
pub fn set_local_endpoint_link(local_endpoint: u16, remote_endpoint: IpEndpoint) {
    let mut lock = LOCAL_ENDPOINT_MAP.lock();
    if !lock.contains_key(&local_endpoint) {
        // warn!("local endpoint not exists");
        return;
    }
    // if lock.get(&local_endpoint).unwrap().is_some() {
    //     warn!("local endpoint has been occupied");
    // }
    lock.insert(local_endpoint, Some(remote_endpoint));
}

pub fn remove_local_endpoint(local_endpoint: u16) {
    let mut lock = LOCAL_ENDPOINT_MAP.lock();
    let remote_endpoint = lock.remove(&local_endpoint).unwrap_or_else(|| {
        // warn!("Local endpoint {local_endpoint} has no remote endpoint");
        return None;
    });
    if remote_endpoint.is_some() {
        debug!(
            "connect to remote {} is closed, release port {}",
            remote_endpoint.unwrap(),
            local_endpoint
        );
    }
}
