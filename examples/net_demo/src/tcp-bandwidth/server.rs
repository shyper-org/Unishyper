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

extern "C" fn netdemo_server(_arg: usize) {
    println!("Server for latency test running, listening for connection on 0.0.0.0:4444");
    let listener = TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        4444,
    ))
    .unwrap();

    let n_bytes = 1048576;
    let n_rounds = 100;
    let tot_n_bytes = (n_rounds * n_bytes) as u64;

    println!("********network  bind ******");

    let (stream, socket_addr) = listener.accept().unwrap();

    println!(
        "Connection established with {:?}! socket addr {:?}",
        stream.peer_addr().unwrap(),
        socket_addr
    );

    let mut buf = vec![0; n_bytes];
    let mut active = true;
    let mut tot_bytes: u64 = 0;
    let mut tot_bytes_stable: u64 = 0;
    let mut tot_time_stable: u64 = 0;
    let mut _i = 0;

    let mut start = current_us() as u64;
    let mut end = 0 as u64;
    while active {
        let recv = stream.read(&mut buf).unwrap();
        if recv > 0 {
            end = current_us() as u64;
            let duration = end - start;

            // Capture measures
            tot_bytes += recv as u64;
            if tot_bytes > tot_n_bytes / 3 && tot_bytes < (tot_n_bytes * 2) / 3 {
                tot_bytes_stable += recv as u64;
                tot_time_stable += duration;
            }
            println!(
                "round {}, recv {} bytes in {} us, tot_bytes {}",
                _i, recv, duration, tot_bytes
            );
        } else {
            active = false;
        }
        if tot_bytes == tot_n_bytes {
            break;
        }
        _i += 1;
        start = end;
    }
    print!(
        "Receive total {} Bytes, stable {} Bytes, stable connection for {} us\n",
        tot_bytes, tot_bytes_stable, tot_time_stable
    );
    println!(
        "Available approximated bandwidth: {} MB/s",
        tot_bytes_stable / tot_time_stable
    );
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
