#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use alloc::vec;
use no_std_net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

use net_demo::connection;

use unishyper::*;

#[macro_use]
extern crate alloc;

extern "C" fn latency_server(_arg: usize) {
    println!("Server for latency test running, listening for connection on 0.0.0.0:4444");

    let listener =
        TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 4444)).unwrap();

    println!("********network  bind ******");

    let (mut stream, socket_addr) = listener.accept().unwrap();

    println!(
        "Connection established with {:?}! socket addr {:?}",
        stream.peer_addr().unwrap(),
        socket_addr
    );

    let n_bytes = 1;
    let n_rounds = 100;
    let mut buf = vec![0; n_bytes];

    stream
        .set_nodelay(true)
        .expect("Can't set no_delay to true");

    for _i in 0..(n_rounds * 2) {
        connection::receive_message(n_bytes, &mut stream, &mut buf);
        connection::send_message(n_bytes, &mut stream, &buf);
    }
    println!("Done exchanging stuff")
}

#[no_mangle]
fn main() {
    println!("********enter net demo server main******");

    network_init();

    println!("********network_init finished ******");

    let tid = thread_spawn(latency_server, 123);
    println!("Spawn user network latency_server thread with id {}", tid);

    exit();
}
