#[cfg(feature = "tcp")]
mod tcp;

#[cfg(feature = "tcp")]
pub use tcp::*;

mod tls;
pub use tls::*;

mod fs;
pub use fs::*;

use crate::libs::thread::Tid;
use crate::libs::thread::thread_exit;

#[no_mangle]
pub extern "C" fn shyper_malloc(size: usize, align: usize) -> *mut u8 {
    if true {
        crate::mm::heap::malloc(size, align)
    } else {
        crate::mm::allocate(size).map_or(core::ptr::null_mut() as *mut u8, |vaddr| {
            vaddr.as_mut_ptr::<u8>()
        })
    }
}

#[no_mangle]
pub extern "C" fn shyper_realloc(
    _ptr: *mut u8,
    _size: usize,
    _align: usize,
    _new_size: usize,
) -> *mut u8 {
    unimplemented!("shyper realloc unimplemented");
}

use crate::mm::address::VAddr;
#[no_mangle]
pub extern "C" fn shyper_free(ptr: *mut u8, size: usize, align: usize) {
    if true {
        crate::mm::heap::free(ptr, size, align)
    } else {
        crate::mm::deallocate(VAddr::new_canonical(ptr as usize))
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct timespec {
    /// seconds
    pub tv_sec: i64,
    /// nanoseconds
    pub tv_nsec: i64,
}

#[no_mangle]
pub extern "C" fn shyper_futex_wait(
    _address: *mut u32,
    _expected: u32,
    _timeout: *const timespec,
    _flags: u32,
) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_futex_wake(_address: *mut u32, _count: i32) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_getpid() -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_exit(arg: i32) {
    debug!("main thread exit with arg {}", arg);
    thread_exit();
}

#[no_mangle]
pub extern "C" fn shyper_abort() {
    info!("shyper system shutdown, currently not supported, just exit currently thread");
    thread_exit();
}

#[no_mangle]
pub extern "C" fn shyper_usleep(_usecs: u64) {}

#[no_mangle]
pub extern "C" fn shyper_spawn(
    _id: *mut Tid,
    _func: extern "C" fn(usize),
    _arg: usize,
    _prio: u8,
    _core_id: isize,
) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_spawn2(
    _func: extern "C" fn(usize),
    _arg: usize,
    _prio: u8,
    _stack_size: usize,
    _core_id: isize,
) -> Tid {
    0 as Tid
}

#[no_mangle]
pub extern "C" fn shyper_join(_id: Tid) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_yield() {}

#[no_mangle]
pub extern "C" fn shyper_clock_gettime(_clock_id: u64, _tp: *mut timespec) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_network_init() -> i32 {
    debug!("Unishyper network init");
    0
}
