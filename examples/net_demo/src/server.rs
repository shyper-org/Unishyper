#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use alloc::vec;
use no_std_net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

use rust_shyper_os::*;

#[macro_use]
extern crate alloc;

extern "C" fn netdemo_server(arg: usize) {
    let core_id = core_id();
    println!(
            "\n**************************\n netdemo_server, core {} arg {}\n**************************\n",
            core_id,
            arg
        );
    let listener = TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        4444,
    ))
    .unwrap();

    let n_bytes = 1048576;
    let n_rounds = 1000;
    let tot_bytes = n_rounds * n_bytes;

    println!("********network  bind ******");

    let (mut stream, socket_addr) = listener.accept().unwrap();

    println!(
        "Connection established with {:?}! socket addr {:?}",
        stream.peer_addr().unwrap(),
        socket_addr
    );

    let mut buf = vec![0; n_bytes];

    let start = current_ms() as f64;
    for _i in 0..n_rounds {
        // println!("round {}", _i);
        stream.read_exact(&mut buf).unwrap();
        // match stream.read(&mut buf) {
        //     Ok(n) => {
        //         println!("round {} read {} bytes", _i, n);
        //     }
        //     Err(e) => {
        //         println!("server read error {}", e);
        //     }
        // }
    }
    let end = current_ms() as f64;
    let total_seconds = (end - start) / 1000.0f64;

    println!(
        "Sent in total {} KBytes, total seconds {}, start {}, end {}",
        tot_bytes / 1024,
        total_seconds,
        start,
        end
    );
    println!(
        "Available approximated bandwidth: {} Mbit/s",
        (tot_bytes as f64 * 8.0f64) / (1024.0f64 * 1024.0f64 * total_seconds)
    );
    // use alloc::string::String;
    // let s = String::from_utf8(buf).expect("Found invalid UTF-8");
    // println!("TCP Connection read, get \"{}\" from client", s);
    loop {}
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
