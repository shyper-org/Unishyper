#![no_std]
#![no_main]
// #![feature(restricted_std)]

#[cfg(target_os = "shyper")]
use unishyper as _;

extern crate alloc;

use unishyper::*;

use alloc::vec;
use alloc::string::String;

#[no_mangle]
pub fn main() {
    let heart = vec![240, 159, 146, 151];
    println!(
        "Hello from Unishyper {}",
        String::from_utf8(heart).unwrap_or_default()
    );
}
