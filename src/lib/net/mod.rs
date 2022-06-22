// tcp api
pub mod tcplistener;
pub mod tcpstream;
// network implement on smoltcp
pub mod interface;
mod device;
mod executor;
mod waker;

pub use smoltcp::wire::IpAddress;
pub use smoltcp::socket::SocketHandle as Handle;

pub fn init() -> i32 {
	info!("network init!\n");
    interface::network_init();
    0
}

