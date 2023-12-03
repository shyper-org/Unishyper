#![feature(allocator_api)]

#[cfg(target_os = "shyper")]
use unishyper as _;

mod test;

fn args() -> std::env::Args {
    std::env::args()
}

fn main() {
    println!("get args {:?}", args());

    println!("hello world!");
    test::run_tests();
}
