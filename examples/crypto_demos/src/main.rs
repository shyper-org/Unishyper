#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

extern crate alloc;

extern crate ring;
extern crate data_encoding;

use unishyper::*;
// use unishyper::shyperstd as std;

#[allow(unused)]
mod hmac;
#[allow(unused)]
mod pbkdf2;
#[allow(unused)]
mod sha256;

#[allow(unused)]
mod hmac_verify_hack;

#[no_mangle]
fn main() {
    println!("Hello! Unishyper crypto demos based on ring[https://crates.io/crates/ring]");

    sha256::sha256_test();

    // hmac::hmac_sign_and_verify();
    hmac::hmac_sign_and_verify_one_shot();

    // pbkdf2::pbkdf2_test();

    hmac_verify_hack::hmac_sign_verify();
}
