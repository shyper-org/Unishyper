#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use alloc::vec;

use net_tcp_bench::connection;

use unishyper::*;
use unishyper::shyperstd as std;

use std::net::{TcpListener, TcpStream};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

#[macro_use]
extern crate alloc;

extern "C" fn latency_server(_arg: usize) {
    let n_bytes = if let Some(k) = option_env!("K") {
        k.parse::<usize>().unwrap()
    } else {
        1
    };

    let n_rounds = if let Some(r) = option_env!("R") {
        r.parse::<usize>().unwrap()
    } else {
        1000
    };

    println!("Server for latency test running {n_bytes} bytes each for {n_rounds} rounds, listening for connection on 0.0.0.0:4444");

    irq_disable();

    let listener =
        TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 4444)).unwrap();

    println!("********network  bind ******");

    let (mut stream, socket_addr) = listener.accept().unwrap();

    println!(
        "Connection established with {:?}! socket addr {:?}",
        stream.peer_addr().unwrap(),
        socket_addr
    );

    let mut buf = vec![0; n_bytes];

    stream
        .set_nodelay(true)
        .expect("Can't set no_delay to true");

    for _i in 0..(n_rounds * 2) {
        connection::receive_message(n_bytes, &mut stream, &mut buf);
        connection::send_message(n_bytes, &mut stream, &buf);
    }

    irq_disable();
    println!("Done exchanging stuff");
    loop {
        irq_disable();
    }
}

#[no_mangle]
fn main() {
    println!("********enter net demo server main******");
    let tid = thread_spawn(latency_server, 123);
    println!("Spawn user network latency_server thread with id {}", tid);
}
