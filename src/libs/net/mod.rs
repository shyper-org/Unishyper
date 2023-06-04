// tcp api
pub mod tcplistener;
pub mod tcpstream;
// network implement on smoltcp
mod device;
mod executor;
pub mod interface;

pub use smoltcp::wire::IpAddress;
pub use interface::*;

pub fn init() -> i32 {
    interface::network_init();
    0
}
