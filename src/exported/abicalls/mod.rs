#[cfg(feature = "tcp")]
mod tcp;

#[cfg(feature = "tcp")]
pub use tcp::*;

mod tls;
pub use tls::*;

mod fs;
pub use fs::*;

use core::ffi::c_void;

use crate::libs::thread::Tid;
use crate::libs::thread::thread_exit;

#[no_mangle]
pub extern "C" fn shyper_malloc(_size: usize, _align: usize) -> *mut u8 {
    0 as *mut u8
}

#[no_mangle]
pub extern "C" fn shyper_realloc(
    _ptr: *mut u8,
    _size: usize,
    _align: usize,
    _new_size: usize,
) -> *mut u8 {
    0 as *mut u8
}

#[no_mangle]
pub extern "C" fn shyper_free(_ptr: *mut u8, _size: usize, _align: usize) {}

#[no_mangle]
pub extern "C" fn shyper_init_queue(_ptr: usize) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_notify(_id: usize, _count: i32) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_add_queue(_id: usize, _timeout_ns: i64) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_wait(_id: usize) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_destroy_queue(_id: usize) -> i32 {
    0
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
pub extern "C" fn shyper_sem_init(_sem: *mut *const c_void, _value: u32) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_sem_destroy(_sem: *const c_void) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_sem_post(_sem: *const c_void) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_sem_trywait(_sem: *const c_void) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_sem_timedwait(_sem: *const c_void, _ms: u32) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_recmutex_init(_recmutex: *mut *const c_void) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_recmutex_destroy(_recmutex: *const c_void) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_recmutex_lock(_recmutex: *const c_void) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_recmutex_unlock(_recmutex: *const c_void) -> i32 {
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

#[no_mangle]
pub extern "C" fn shyper_block_current_task() {}

#[no_mangle]
pub extern "C" fn shyper_block_current_task_with_timeout(_timeout: u64) {}

#[no_mangle]
pub extern "C" fn shyper_wakeup_task(_tid: Tid) {}
