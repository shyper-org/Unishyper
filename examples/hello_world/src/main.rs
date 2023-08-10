#![feature(restricted_std)]

#[cfg(target_os = "shyper")]
use unishyper as _;

fn main() {
	println!("hello world!");
}
