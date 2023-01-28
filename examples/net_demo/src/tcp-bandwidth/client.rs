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

extern "C" fn netdemo_client(_arg: usize) {
    println!("Connecting to the server 10.0.0.2...");

    let n_bytes = 1048576;
    let n_rounds = 100;
    let tot_n_bytes = (n_bytes * n_rounds) as u64;

    println!(
        "client send {} rounds for {} bytes, total {} bytes",
        n_rounds, n_bytes, tot_n_bytes
    );

    if let Ok(mut stream) = TcpStream::connect(SocketAddr::new(
        // IpAddr::V4(Ipv4Addr::new(192, 168, 106, 140)),
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        4444,
    )) {
        println!("Connection established! Ready to send...");

        // Create a buffer of 0s, size n_bytes, to be sent over multiple times
        let buf = vec![0; n_bytes];
        let progress_tracking_percentage = n_rounds / 100;

        for i in 0..n_rounds {
            connection::send_message(n_bytes, &mut stream, &buf);
            // match stream.write(&buf) {
            //     Ok(_n) => {
            //         // println!("round {}, write n {} bytes", i, _n);
            //     }
            //     Err(err) => panic!("crazy stuff happened while sending {}", err),
            // }
            if i % progress_tracking_percentage == 0 {
                println!("{}% completed", i / progress_tracking_percentage);
            }
        }

        stream
            .shutdown(net::Shutdown::Both)
            .expect("shutdown call failed");

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
