use no_std_net::{IpAddr, SocketAddr};
use smoltcp::wire::{IpAddress, IpEndpoint, Ipv4Address};

pub const fn core_ipaddr_to_ipaddress(ip: IpAddr) -> IpAddress {
    match ip {
        IpAddr::V4(ipv4) => IpAddress::Ipv4(Ipv4Address(ipv4.octets())),
        _ => panic!("IPv6 not supported"),
    }
}

pub const fn ipaddress_to_core_ipaddr(ip: IpAddress) -> IpAddr {
    match ip {
        IpAddress::Ipv4(ipv4) => IpAddr::V4(unsafe { core::mem::transmute(ipv4.0) }),
        _ => panic!("IPv6 not supported"),
    }
}

pub const fn core_socketaddr_to_ipendpoint(addr: SocketAddr) -> IpEndpoint {
    IpEndpoint {
        addr: core_ipaddr_to_ipaddress(addr.ip()),
        port: addr.port(),
    }
}

pub const fn ipendpoint_to_core_socketaddr(addr: IpEndpoint) -> SocketAddr {
    SocketAddr::new(ipaddress_to_core_ipaddr(addr.addr), addr.port)
}

pub fn is_unspecified(ip: IpAddress) -> bool {
    ip.as_bytes() == [0, 0, 0, 0]
}

pub const UNSPECIFIED_ENDPOINT: IpEndpoint = IpEndpoint::UNSPECIFIED;