#![no_std]
// error: requires `start` lang_item
#![no_main]
// error[E0658]: use of unstable library feature 'format_args_nl': `format_args_nl` is only for internal language use and is subject to change
// help: add `#![feature(format_args_nl)]` to the crate attributes to enable
// note: this error originates in the macro `println` (in Nightly builds, run with -Z macro-backtrace for more info)
#![feature(format_args_nl)]

extern crate alloc;

use core::hint::black_box;
// use core::ffi::c_void;
use alloc::vec;

use unishyper::*;
use unishyper::shyperstd as std;

use unishyper::libs::string::memcpy;
use unishyper::libs::string::memset;

// use std::time::Instant;
use std::mm;

const NR_RUNS: usize = 100000;
// const TRACE_STEP: usize = 10;

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memcpy_builtin(n: usize) {
    let v1 = vec![1u8; n];
    let mut v2 = vec![0u8; n];

    let mut results = vec![];

    for i in 0..(NR_RUNS * 2) {
        let src: &[u8] = black_box(&v1);
        let dst: &mut [u8] = black_box(&mut v2);

        let start = unsafe { core::arch::x86_64::_rdtsc() };
        dst.copy_from_slice(src);
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memcpy_builtin, round [{}] copy_from_slice {} cycles",
        //         i,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memcpy_builtin:\t\t  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memcpy_builtin_zone_shared(n: usize) {
    let n = align_up!(n, 4096);
    let v1 = mm::allocate(n).as_mut_ptr::<u8>();
    let v2 = mm::allocate(n).as_mut_ptr::<u8>();

    let v1 = unsafe { core::slice::from_raw_parts(v1, n) };
    let v2 = unsafe { core::slice::from_raw_parts_mut(v2, n) };

    let mut results = vec![];

    for i in 0..(NR_RUNS * 2) {
        let src: &[u8] = black_box(v1);
        let dst: &mut [u8] = black_box(v2);
        let start = unsafe { core::arch::x86_64::_rdtsc() };
        dst.copy_from_slice(src);
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memcpy_builtin_zone_shared, round [{}] copy_from_slice {} cycles",
        //         i,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memcpy_builtin_zone_shared:  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memcpy_builtin_zone_protected(n: usize) {
    let n = align_up!(n, 4096);

    let mut results = vec![];

    let v1 = mm::allocate_zone(n).as_mut_ptr::<u8>();
    let v2 = mm::allocate_zone(n).as_mut_ptr::<u8>();

    let v1 = unsafe { core::slice::from_raw_parts(v1, n) };
    let v2 = unsafe { core::slice::from_raw_parts_mut(v2, n) };

    // let now = Instant::now();
    for i in 0..(NR_RUNS * 2) {
        let src: &[u8] = black_box(v1);
        let dst: &mut [u8] = black_box(v2);
        let start = unsafe { core::arch::x86_64::_rdtsc() };
        dst.copy_from_slice(src);
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memcpy_builtin_zone_protected, round [{}] copy_from_slice {} cycles",
        //         i,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memcpy_builtin_zone_protected:  {} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memset_builtin(n: usize) {
    let mut results = vec![];

    let mut v1 = vec![0u8; n];

    for i in 0..(NR_RUNS * 2) {
        let dst: &mut [u8] = black_box(&mut v1);
        let val: u8 = black_box(27);

        let start = unsafe { core::arch::x86_64::_rdtsc() };
        for b in dst {
            *b = val;
        }
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memset_builtin, round [{}] set {} bytes within {} cycles",
        //         i,n,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memset_builtin:\t\t  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memset_builtin_zone_shared(n: usize) {
    let n = align_up!(n, 4096);

    let mut results = vec![];

    let v1 = mm::allocate(n).as_mut_ptr::<u8>();
    let v1 = unsafe { core::slice::from_raw_parts_mut(v1, n) };

    // let now = Instant::now();
    for i in 0..(NR_RUNS * 2) {
        let dst: &mut [u8] = black_box(v1);
        let val: u8 = black_box(27);

        let start = unsafe { core::arch::x86_64::_rdtsc() };
        for b in dst {
            *b = val;
        }
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memset_builtin_zone_shared, round [{}] set {} bytes within {} cycles",
        //         i,n,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memset_builtin_zone_shared:  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memset_builtin_zone_protected(n: usize) {
    let n = align_up!(n, 4096);

    let mut results = vec![];

    let v1 = mm::allocate_zone(n).as_mut_ptr::<u8>();
    let v1 = unsafe { core::slice::from_raw_parts_mut(v1, n) };

    // let now = Instant::now();
    for i in 0..(NR_RUNS * 2) {
        let dst: &mut [u8] = black_box(v1);
        let val: u8 = black_box(27);

        let start = unsafe { core::arch::x86_64::_rdtsc() };
        for b in dst {
            *b = val;
        }
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memset_builtin_zone_protected, round [{}] set {} bytes within {} cycles",
        //         i,n,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memset_builtin_zone_protected:  {} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memcpy_rust(n: usize) {
    let v1 = vec![1u8; n];
    let mut v2 = vec![0u8; n];
    let mut results = vec![];

    for i in 0..(NR_RUNS * 2) {
        let src: &[u8] = black_box(&v1[0..]);
        let dst: &mut [u8] = black_box(&mut v2[0..]);
        let start = unsafe { core::arch::x86_64::_rdtsc() };
        unsafe {
            memcpy(
                dst.as_mut_ptr(), // as *mut c_void,
                src.as_ptr(),     // as *mut c_void,
                n,
            );
        }
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i %(NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memcpy_rust, round [{}] memcpy {} cycles",
        //         i,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memcpy_rust:\t\t  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

#[allow(unused)]
fn memcpy_rust_zone_shared(n: usize) {
    let n = align_up!(n, 4096);
    let mut results = vec![];

    let v1 = mm::allocate(n).as_mut_ptr::<u8>();
    let v2 = mm::allocate(n).as_mut_ptr::<u8>();

    let v1 = unsafe { core::slice::from_raw_parts(v1, n) };
    let v2 = unsafe { core::slice::from_raw_parts_mut(v2, n) };

    // let now = Instant::now();
    for i in 0..(NR_RUNS * 2) {
        let src: &[u8] = black_box(&v1[0..]);
        let dst: &mut [u8] = black_box(&mut v2[0..]);
        let start = unsafe { core::arch::x86_64::_rdtsc() };
        unsafe {
            memcpy(
                dst.as_mut_ptr(), // as *mut c_void,
                src.as_ptr(),     // as *mut c_void,
                n,
            );
        }
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memcpy_rust_zone_shared, round [{}] memcpy {} cycles",
        //         i,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memcpy_rust_zone_shared:  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

#[allow(unused)]
fn memcpy_rust_zone_protected(n: usize) {
    let n = align_up!(n, 4096);
    let mut results = vec![];

    let v1 = mm::allocate_zone(n).as_mut_ptr::<u8>();
    let v2 = mm::allocate_zone(n).as_mut_ptr::<u8>();

    let v1 = unsafe { core::slice::from_raw_parts(v1, n) };
    let v2 = unsafe { core::slice::from_raw_parts_mut(v2, n) };

    // let now = Instant::now();
    for i in 0..(NR_RUNS * 2) {
        let src: &[u8] = black_box(&v1[0..]);
        let dst: &mut [u8] = black_box(&mut v2[0..]);
        let start = unsafe { core::arch::x86_64::_rdtsc() };
        unsafe {
            memcpy(
                dst.as_mut_ptr(), // as *mut c_void,
                src.as_ptr(),     // as *mut c_void,
                n,
            );
        }
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memcpy_rust_zone_protected, round [{}] memcpy {} cycles",
        //         i,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memcpy_rust_zone_protected:  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memset_rust(n: usize) {
    let mut v1 = vec![0u8; n];
    let mut results = vec![];
    for i in 0..(NR_RUNS * 2) {
        let dst: &mut [u8] = black_box(&mut v1[0..]);
        let val = black_box(27);

        let start = unsafe { core::arch::x86_64::_rdtsc() };
        unsafe {
            memset(dst.as_mut_ptr(), val, n);
        }
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memset_rust, round [{}] set {} bytes within {} cycles",
        //         i,n,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memset_rust:\t\t  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memset_rust_zone_shared(n: usize) {
    let v1 = mm::allocate(n).as_mut_ptr::<u8>();
    let v1 = unsafe { core::slice::from_raw_parts_mut(v1, n) };
    let mut results = vec![];
    for i in 0..(NR_RUNS * 2) {
        let dst: &mut [u8] = black_box(&mut v1[0..]);
        let val = black_box(27);

        let start = unsafe { core::arch::x86_64::_rdtsc() };
        unsafe {
            memset(dst.as_mut_ptr(), val, n);
        }
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memset_rust_zone_shared, round [{}] set {} bytes within {} cycles",
        //         i,n,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memset_rust_zone_shared:  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

// derived from
// https://github.com/rust-lang/compiler-builtins/blob/master/testcrate/benches/mem.rs
#[allow(unused)]
fn memset_rust_zone_protected(n: usize) {
    let v1 = mm::allocate_zone(n).as_mut_ptr::<u8>();
    let v1 = unsafe { core::slice::from_raw_parts_mut(v1, n) };
    let mut results = vec![];
    for i in 0..(NR_RUNS * 2) {
        let dst: &mut [u8] = black_box(&mut v1[0..]);
        let val = black_box(27);

        let start = unsafe { core::arch::x86_64::_rdtsc() };
        unsafe {
            memset(dst.as_mut_ptr(), val, n);
        }
        let end = unsafe { core::arch::x86_64::_rdtsc() };

        if i >= NR_RUNS {
            results.push(end - start);
        }
        // if i % (NR_RUNS / TRACE_STEP) == 0 || i < 10 {
        //     println!(
        //         "memset_rust_zone_protected, round [{}] set {} bytes within {} cycles",
        //         i,n,
        //         end - start
        //     );
        // }
    }

    let mut sum = 0;
    for result in results {
        sum += result;
    }

    println!(
        "memset_rust_zone_protected:  \t{} block size, avg: {:.3} cycles",
        n,
        sum as f64 / NR_RUNS as f64
    );
}

fn bench_with_size(n :usize) {
    memcpy_builtin(n);
    memcpy_builtin_zone_shared(n);
    memcpy_builtin_zone_protected(n);
    memset_builtin(n);
    memset_builtin_zone_shared(n);
    memset_builtin_zone_protected(n);
    memcpy_rust(n);
    memcpy_rust_zone_shared(n);
    memcpy_rust_zone_protected(n);
    memset_rust(n);
    memset_rust_zone_shared(n);
    memset_rust_zone_protected(n);
}

#[no_mangle]
fn main() {
    println!("memory test bench on Unishyper");
    // bench_with_size(4096);

    memcpy_builtin(1048576);
    memset_builtin(1048576);
    memcpy_rust(1048576);
    memset_rust(1048576);

    println!("memory test bench finished");

}
