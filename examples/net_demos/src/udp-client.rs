#![no_std]
#![no_main]
#![feature(format_args_nl)]

use unishyper::*;
use unishyper::shyperstd as std;

use std::io;
use std::net::{ToSocketAddrs, UdpSocket, Ipv4Addr, SocketAddr};

const LOCAL_IP: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
const LOCAL_PORT: u16 = 5555;

fn receive_loop() -> io::Result<()> {
    let addr = (LOCAL_IP, LOCAL_PORT)
        .to_socket_addrs()
        .map_err(|_| "ToSocketAddrError")?
        .next()
        .unwrap();
    let socket = UdpSocket::bind(addr)?;
    println!("listen on: {}", socket.local_addr().unwrap());
    let mut buf = [0u8; 1024];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("recv: {}Bytes from {}", size, addr);
                let mid = core::str::from_utf8(&buf).unwrap();
                println!("{}", mid);
                let mid = ["response_", mid].join("");
                socket.send_to(mid.as_bytes(), addr)?;
                buf = [0u8; 1024];
            }
            Err(e) => return Err(e),
        };
    }
}

#[no_mangle]
fn main() {
    println!("Unishyper Udp client demo");
    // receive_loop().expect("test udp server failed");

    // let addr = (LOCAL_IP, LOCAL_PORT)
    //     .to_socket_addrs()
    //     .map_err(|_| "ToSocketAddrError")?
    //     .next()
    //     .unwrap();
    let addr = SocketAddr::from(([10, 0, 0, 2], 5555));
    let target_addr = SocketAddr::from(([10, 0, 0, 1], 5555));
    let socket = UdpSocket::bind(addr).expect("bind failed");

    for i in 0..10 {
        socket
            .send_to("hello".as_bytes(), target_addr)
            .expect("send to failed");
    }

    let mut buf = [0u8; 1024];

    loop {
        socket
            .send_to("exit".as_bytes(), target_addr)
            .expect("send to failed");
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("recv: {}Bytes from {}", size, addr);
                break;
                buf = [0u8; 1024];
            }
            Err(e) => {break;},
        };
    }
}
