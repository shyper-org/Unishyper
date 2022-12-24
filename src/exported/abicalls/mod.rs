#[cfg(feature = "tcp")]
mod tcp;

#[cfg(feature = "tcp")]
pub use tcp::*;

mod tls;
pub use tls::*;

use core::ffi::c_void;

use crate::libs::thread::Tid;
use crate::libs::thread::thread_exit;

#[no_mangle]
pub extern "C" fn shyper_rand() -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_srand(_seed: u32) {}

#[no_mangle]
pub extern "C" fn shyper_secure_rand32(_value: *mut u32) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_secure_rand64(_value: *mut u64) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_get_processor_count() -> usize {
    0
}

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

#[no_mangle]
pub extern "C" fn shyper_open(name: *const u8, flags: i32, mode: i32) -> i32 {
    let path = unsafe { core::ffi::CStr::from_ptr(name as _) }.to_str().unwrap();
	crate::libs::fs::open(path, flags, mode)
}

#[no_mangle]
pub extern "C" fn shyper_read(fd: i32, buf: *mut u8, len: usize) -> isize {
    if fd > 2 {
        // Normal file
        if cfg!(feature = "fs") {
            crate::libs::fs::read(fd, buf, len)
        } else {
            warn!(
                "\"fs\" feature is not enabled for shyper, read from fd {} failed",
                fd
            );
            0 as isize
        }
    } else {
        warn!("read from stdin/err/out is unimplemented, returning -1");
        -1 as isize
    }
}

#[no_mangle]
pub extern "C" fn shyper_write(fd: i32, buf: *const u8, len: usize) -> isize {
    if fd > 2 {
        // Normal file
        if cfg!(feature = "fs") {
            crate::libs::fs::write(fd, buf, len)
        } else {
            warn!(
                "\"fs\" feature is not enabled for shyper, write to fd {} failed",
                fd
            );
            0 as isize
        }
    } else {
        // stdin/err/out all go to console
        let buf = unsafe { core::slice::from_raw_parts(buf, len) };
        crate::libs::print::print_byte(buf);
        len as isize
    }
}

#[no_mangle]
pub extern "C" fn shyper_lseek(fd: i32, offset: isize, whence: i32) -> isize {
    if fd > 2 {
        // Normal file
        if cfg!(feature = "fs") {
            crate::libs::fs::lseek(fd, offset, whence)
        } else {
            warn!(
                "\"fs\" feature is not enabled for shyper, seek fd {} failed",
                fd
            );
            0 as isize
        }
    } else {
        warn!("seek stdin/err/out is unimplemented, returning -1");
        -1 as isize
    }
}

#[no_mangle]
pub extern "C" fn shyper_close(fd: i32) -> i32 {
    if fd > 2 {
        // Normal file
        if cfg!(feature = "fs") {
            crate::libs::fs::close(fd)
        } else {
            warn!(
                "\"fs\" feature is not enabled for shyper, close fd {} failed",
                fd
            );
            0 as i32
        }
    } else {
        // we don't have to close standard descriptors
        0 as i32
    }
}

#[no_mangle]
pub extern "C" fn shyper_unlink(name: *const i8) -> i32 {
    let path = unsafe { core::ffi::CStr::from_ptr(name as _) }.to_str().unwrap();
    crate::libs::fs::unlink(path)
}


#[no_mangle]
pub extern "C" fn shyper_stat(file: *const u8, st: usize) -> i32 {
    crate::libs::fs::stat(file, st)
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
pub extern "C" fn shyper_abort() {}

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

#[no_mangle]
pub extern "C" fn shyper_get_priority() -> u8 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_set_priority(_tid: Tid, _prio: u8) {}
