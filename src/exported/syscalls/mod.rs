mod tcp;

pub use tcp::*;

use core::ffi::c_void;
use crate::libs::thread::Tid;

#[no_mangle]
pub extern "C" fn sys_rand() -> u32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_srand(_seed: u32) {}
#[no_mangle]
pub extern "C" fn sys_secure_rand32(_value: *mut u32) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_secure_rand64(_value: *mut u64) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_get_processor_count() -> usize {
    0
}
#[no_mangle]
pub extern "C" fn sys_malloc(_size: usize, _align: usize) -> *mut u8 {
    0 as *mut u8
}
#[no_mangle]
pub extern "C" fn sys_realloc(_ptr: *mut u8, _size: usize, _align: usize, _new_size: usize) -> *mut u8 {
    0 as *mut u8
}
#[no_mangle]
pub extern "C" fn sys_free(_ptr: *mut u8, _size: usize, _align: usize) {}
#[no_mangle]
pub extern "C" fn sys_init_queue(_ptr: usize) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_notify(_id: usize, _count: i32) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_add_queue(_id: usize, _timeout_ns: i64) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_wait(_id: usize) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_destroy_queue(_id: usize) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_read(_fd: i32, _buf: *mut u8, _len: usize) -> isize {
    0
}
#[no_mangle]
pub extern "C" fn sys_write(_fd: i32, _buf: *const u8, _len: usize) -> isize {
    0
}
#[no_mangle]
pub extern "C" fn sys_close(_fd: i32) -> i32 {
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
pub extern "C" fn sys_futex_wait(
    _address: *mut u32,
    _expected: u32,
    _timeout: *const timespec,
    _flags: u32,
) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_futex_wake(_address: *mut u32, _count: i32) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_sem_init(_sem: *mut *const c_void, _value: u32) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_sem_destroy(_sem: *const c_void) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_sem_post(_sem: *const c_void) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_sem_trywait(_sem: *const c_void) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_sem_timedwait(_sem: *const c_void, _ms: u32) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_recmutex_init(_recmutex: *mut *const c_void) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_recmutex_destroy(_recmutex: *const c_void) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_recmutex_lock(_recmutex: *const c_void) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_recmutex_unlock(_recmutex: *const c_void) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_getpid() -> u32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_exit(_arg: i32) {
}
#[no_mangle]
pub extern "C" fn sys_abort() {
}
#[no_mangle]
pub extern "C" fn sys_usleep(_usecs: u64) {}
#[no_mangle]
pub extern "C" fn sys_spawn(
    _id: *mut Tid,
    _func: extern "C" fn(usize),
    _arg: usize,
    _prio: u8,
    _core_id: isize,
) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_spawn2(
    _func: extern "C" fn(usize),
    _arg: usize,
    _prio: u8,
    _stack_size: usize,
    _core_id: isize,
) -> Tid {
    0 as Tid
}
#[no_mangle]
pub extern "C" fn sys_join(_id: Tid) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_yield() {}
#[no_mangle]
pub extern "C" fn sys_clock_gettime(_clock_id: u64, _tp: *mut timespec) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_open(_name: *const i8, _flags: i32, _mode: i32) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_unlink(_name: *const i8) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_network_init() -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn sys_block_current_task() {}
#[no_mangle]
pub extern "C" fn sys_block_current_task_with_timeout(_timeout: u64) {}
#[no_mangle]
pub extern "C" fn sys_wakeup_task(_tid: Tid) {}
#[no_mangle]
pub extern "C" fn sys_get_priority() -> u8 {
    0
}
#[no_mangle]
pub extern "C" fn sys_set_priority(_tid: Tid, _prio: u8) {}
