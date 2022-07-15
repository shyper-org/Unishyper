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

#[allow(dead_code)]
extern "C" fn netdemo_server(arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
            "\n**************************\n netdemo_server, core {} arg {} curent EL{}\n**************************\n",
            core_id,
            arg,
            crate::arch::Arch::curent_privilege()
        );
    let listener = TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 0, 5, 3)),
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
    println!("TCP Connection read, get {:?}", s);
    loop{}
}

#[allow(dead_code)]
extern "C" fn netdemo_client(arg: usize) {
    let core_id = crate::arch::Arch::core_id();
    println!(
            "\n**************************\n netdemo_client, core {} arg {} curent EL{}\n**************************\n",
            core_id,
            arg,
            crate::arch::Arch::curent_privilege()
        );
    if let Ok(stream) = TcpStream::connect(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 106, 140)),
        4444,
    )) {
        println!("Connection established! Ready to send...");

        // Create a buffer of 0s, size n_bytes, to be sent over multiple times
        let mut buf = vec![0; 1024];
        buf[0] = 0x48;
        buf[1] = 0x45;
        buf[2] = 0x4C;
        buf[3] = 0x4C;
        buf[4] = 0x4F;
        buf[5] = 0x0;

        for _i in 0..5 {
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
    }
    println!("exit");
}

#[no_mangle]
fn main() {
    println!("********enter main******");

    network_init();

    println!("********network_init finished ******");

    thread_spawn(netdemo_client, 123);
    // thread_spawn(netdemo_server, 123);

    exit();
    loop {}
}
