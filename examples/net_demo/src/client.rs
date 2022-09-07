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

extern "C" fn netdemo_client(arg: usize) {
    let core_id = core_id();
    println!(
            "\n**************************\n netdemo_client, core {} arg {}\n**************************\n",
            core_id,
            arg
        );

    let n_bytes = 1048576;
    let n_rounds = 1000;
    if let Ok(stream) = TcpStream::connect(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 106, 140)),
        // IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        4444,
    )) {
        println!("Connection established! Ready to send...");

        // Create a buffer of 0s, size n_bytes, to be sent over multiple times
        let mut buf = vec![0; n_bytes];
        buf[0] = 0x48;
        buf[1] = 0x45;
        buf[2] = 0x4C;
        buf[3] = 0x4C;
        buf[4] = 0x4F;
        buf[5] = 0x0;

        for _i in 0..n_rounds {
            println!("round {}", _i);
            let mut pos = 0;
            while pos < buf.len() {
                let bytes_written = match stream.write(&buf[pos..]) {
                    Ok(len) => len,
                    Err(e) => panic!("encountered IO error: {}", e),
                };
                pos += bytes_written;
            }
        }

        stream.shutdown(2).expect("shutdown call failed");

        println!("Sent everything!");
    } else {
        println!("connect failed");
    }
    // println!("exit");
    loop {}
}

#[no_mangle]
fn main() {
    println!("********enter network demo client main******");

    network_init();

    println!("********network_init finished ******");

    let tid = thread_spawn(netdemo_client, 123);
    println!("Spawn user network client thread with id {}", tid);

    exit();
}
