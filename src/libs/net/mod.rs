// tcp api
pub mod tcplistener;
pub mod tcpstream;
// network implement on smoltcp
pub mod interface;
mod device;
mod executor;

pub use smoltcp::wire::IpAddress;
pub use interface::Handle;

pub fn init() -> i32 {
    interface::network_init();
    0
}

