#![feature(allocator_api)]

#[cfg(target_os = "shyper")]
use unishyper as _;

mod test;

fn main() {
	println!("hello world!");
	test::run_tests();
}