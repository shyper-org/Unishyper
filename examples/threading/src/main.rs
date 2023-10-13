#![no_std]
#![no_main]
#![feature(format_args_nl)]

use core::sync::atomic::{AtomicUsize, Ordering};

use unishyper::*;
use shyperstd::thread;

const NUM_TASKS: usize = 10;
static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn main() {
    for i in 0..NUM_TASKS {
        thread::spawn(move || {
            println!("Hello, task {}! id = {:?}", i, thread::current().id());

            thread::yield_now();

            let _order = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
        });
    }
    println!("Hello, main task!");
    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        thread::yield_now();
    }
    println!("Task yielding tests run OK!");
}
