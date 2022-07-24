#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use alloc::vec;
use no_std_net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use rust_shyper_os::arch::*;
use rust_shyper_os::exported::*;
use rust_shyper_os::*;

#[macro_use]
extern crate alloc;

extern "C" fn netdemo_server(arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
            "\n**************************\n netdemo_server, core {} arg {} curent EL{}\n**************************\n",
            core_id,
            arg,
            crate::arch::Arch::curent_privilege()
        );
    let listener = TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        4444,
    ))
    .unwrap();

    println!("********network  bind ******");

    let (stream, socket_addr) = listener.accept().unwrap();

    println!(
        "Connection established with {:?}! socket addr {:?}",
        stream.peer_addr().unwrap(),
        socket_addr
    );

    let mut buf = vec![0; 1024];
    stream.read(&mut buf).expect("server stream read error");
    use alloc::string::String;
    
    let s = String::from_utf8(buf).expect("Found invalid UTF-8");
    println!("TCP Connection read, get \"{}\" from client", s);
    loop{}
}

#[no_mangle]
fn main() {
    println!("********enter net demo server main******");

    network_init();

    println!("********network_init finished ******");

    let tid = thread_spawn(netdemo_server, 123);
    println!("Spawn user network server thread with id {}", tid);
    
    exit();
}
