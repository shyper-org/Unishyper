#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use alloc::vec;
use alloc::vec::Vec;

use unishyper::*;
use unishyper::shyperstd as std;

use std::net::{TcpListener, TcpStream, Shutdown};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

use net_tcp_bench::connection;

#[macro_use]
extern crate alloc;

extern "C" fn latency_client(_arg: usize) {
    println!("Connecting to the server 10.0.0.2...");

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

    // Create buffers to read/write
    let wbuf: Vec<u8> = vec![0; n_bytes];
    let mut rbuf: Vec<u8> = vec![0; n_bytes];

    let progress_tracking_percentage = (n_rounds * 2) / 100;

    let mut connected = false;

    while !connected {
        match TcpStream::connect(SocketAddr::new(
            // IpAddr::V4(Ipv4Addr::new(192, 168, 106, 140)),
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            4444,
        )) {
            Ok(mut stream) => {
                stream
                    .set_nodelay(true)
                    .expect("Can't set no_delay to true");
                connected = true;

                println!("Connection established! Ready to send...");

                let mut results = vec![];
                let mut max = usize::MIN;
                let mut min = usize::MAX;

                // To avoid TCP slowstart we do double iterations and measure only the second half
                for i in 0..(n_rounds * 2) {
                    let start = current_us();

                    connection::send_message(n_bytes, &mut stream, &wbuf);

                    let send_end = current_us();

                    connection::receive_message(n_bytes, &mut stream, &mut rbuf);

                    let end = current_us();
                    let send = send_end - start;
                    let receive = end - send_end;

                    let duration = end - start;

                    println!(
                        "[{}] duration {} us, send {} us receive {} us",
                        i, duration, send, receive
                    );

                    if i >= n_rounds {
                        results.push(duration);
                        if duration > max {
                            max = duration;
                        }
                        if duration < min {
                            min = duration;
                        }
                    }

                    if i % progress_tracking_percentage == 0 {
                        // Track progress on screen
                        println!(
                            "{}% completed, duration {} us",
                            i / progress_tracking_percentage,
                            duration
                        );
                    }
                }
                stream
                    .shutdown(Shutdown::Both)
                    .expect("shutdown call failed");
                println!("latency max: {}us, min: {}us", max, min);
                println!("results: {:?}", results);
            }
            Err(error) => {
                println!("Couldn't connect to server, retrying... Error {}", error);
            }
        }
    }
}

#[no_mangle]
fn main() {
    println!("********enter network demo client main******");
    let tid = thread_spawn(latency_client, 123);
    println!("Spawn user network latency_client thread with id {}", tid);
}
