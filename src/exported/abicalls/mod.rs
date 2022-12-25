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

/// Interface to allocate memory from system heap.
/// Currently its alloc from shyper's buddy system allocator.
/// We need to make sure if our own mm allocator can be used.
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

/// Interface to deallocate a memory region from the system heap.
/// We need to make sure if our own mm allocator can be used.
#[no_mangle]
pub extern "C" fn shyper_free(ptr: *mut u8, size: usize, align: usize) {
    if true {
        crate::mm::heap::free(ptr, size, align)
    } else {
        use crate::mm::address::VAddr;
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
    crate::libs::thread::current_thread_id() as u32
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
pub extern "C" fn shyper_usleep(usecs: u64) {
    crate::libs::thread::thread_block_current_with_timeout_us(usecs as usize)
}

#[no_mangle]
pub extern "C" fn shyper_spawn(
    id: *mut Tid,
    func: extern "C" fn(usize),
    arg: usize,
    _prio: u8,
    selector: isize,
) -> i32 {
    let new_id = crate::libs::thread::thread_spawn_on_core(func, arg, selector);
    if !id.is_null() {
        unsafe {
            *id = new_id;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn shyper_spawn2(
    func: extern "C" fn(usize),
    arg: usize,
    _prio: u8,
    _stack_size: usize,
    selector: isize,
) -> Tid {
    crate::libs::thread::thread_spawn_on_core(func, arg, selector)
}

#[no_mangle]
pub extern "C" fn shyper_join(_id: Tid) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn shyper_yield() {
    crate::libs::thread::thread_yield()
}

fn microseconds_to_timespec(microseconds: usize, result: &mut timespec) {
	result.tv_sec = (microseconds / 1_000_000) as i64;
	result.tv_nsec = ((microseconds % 1_000_000) * 1000) as i64;
}

#[no_mangle]
pub extern "C" fn shyper_clock_gettime(clock_id: u64, tp: *mut timespec) -> i32 {
    use crate::libs::timer::{CLOCK_REALTIME, CLOCK_MONOTONIC, current_us, boot_time};
    assert!(
		!tp.is_null(),
		"shyper_clock_gettime called with a zero tp parameter"
	);
    let result = unsafe { &mut *tp };
    match clock_id {
		CLOCK_REALTIME | CLOCK_MONOTONIC => {
			let mut microseconds = current_us();

			if clock_id == CLOCK_REALTIME {
				microseconds += boot_time();
			}

			microseconds_to_timespec(microseconds, result);
			0
		}
		_ => {
			debug!(
				"Called shyper_clock_gettime for unsupported clock {}",
				clock_id
			);
			-1 as i32
		}
	}
}

#[no_mangle]
pub extern "C" fn shyper_network_init() -> i32 {
    debug!("Unishyper network init");
    0
}
