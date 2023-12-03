#[cfg(feature = "net")]
mod tcp;

#[cfg(feature = "net")]
pub use tcp::*;

#[cfg(feature = "net")]
mod udp;

#[cfg(feature = "net")]
pub use udp::*;

mod tls;
pub use tls::*;

mod fs;
pub use fs::*;

mod cmath;
pub use cmath::*;

pub use crate::libs::string::{memcmp, memmove, memcpy, memset, strlen};

use crate::libs::thread::thread_exit;

/// Interface to allocate memory from system heap.
/// Currently its alloc from shyper's buddy system allocator.
/// We need to make sure if our own mm allocator can be used.
#[no_mangle]
pub extern "C" fn shyper_malloc(size: usize, align: usize) -> *mut u8 {
    if true {
        crate::mm::heap::malloc(size, align)
    } else {
        crate::mm::allocate(size, false).map_or(core::ptr::null_mut() as *mut u8, |vaddr| {
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
pub struct Timespec {
    /// seconds
    pub tv_sec: i64,
    /// nanoseconds
    pub tv_nsec: i64,
}

use core::sync::atomic::AtomicU32;

pub(crate) fn timespec_to_microseconds(time: Timespec) -> Option<usize> {
    usize::try_from(time.tv_sec)
        .ok()
        .and_then(|secs| secs.checked_mul(1_000_000))
        .and_then(|millions| millions.checked_add(usize::try_from(time.tv_nsec).ok()? / 1000))
}

#[no_mangle]
pub extern "C" fn shyper_futex_wait(
    address: *mut u32,
    expected: u32,
    timeout: *const Timespec,
    flags: u32,
) -> i32 {
    let address = unsafe { &*(address as *const AtomicU32) };
    let timeout = if timeout.is_null() {
        None
    } else {
        match timespec_to_microseconds(unsafe { timeout.read() }) {
            t @ Some(_) => t,
            None => return -1,
        }
    };
    let flags = match crate::libs::synch::futex::Flags::from_bits(flags) {
        Some(flags) => flags,
        None => return -1,
    };
    crate::libs::synch::futex::futex_wait(address, expected, timeout, flags)
}

#[no_mangle]
pub extern "C" fn shyper_futex_wake(address: *mut u32, count: i32) -> i32 {
    if address.is_null() {
        return -1;
    }

    let address = unsafe { &*(address as *const AtomicU32) };
    crate::libs::synch::futex::futex_wake(address, count)
}

#[no_mangle]
pub extern "C" fn shyper_getpid() -> u32 {
    crate::libs::thread::current_thread_id().as_u64() as u32
}

#[no_mangle]
pub extern "C" fn shyper_exit(arg: i32) {
    debug!("main thread exit with arg {}", arg);
    thread_exit();
}

#[no_mangle]
pub extern "C" fn shyper_abort() {
    info!("shyper_abort: currently not supported, just exit currently thread");
    crate::arch::irq::disable();
    loop {}
    // thread_exit();
}

#[no_mangle]
pub extern "C" fn shyper_usleep(usecs: u64) {
    crate::libs::thread::thread_block_current_with_timeout_us(usecs as usize)
}

#[no_mangle]
pub extern "C" fn shyper_spawn(
    id: *mut u32,
    func: extern "C" fn(usize),
    arg: usize,
    _prio: u8,
    selector: isize,
) -> i32 {
    let new_id = crate::libs::thread::thread_spawn_on_core(func, arg, selector);
    if !id.is_null() {
        unsafe {
            *id = new_id.as_u64() as u32;
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
) -> u32 {
    crate::libs::thread::thread_spawn_on_core(func, arg, selector).as_u64() as u32
}

#[no_mangle]
pub extern "C" fn shyper_join(id: u32) -> i32 {
    crate::libs::thread::thread_join((id as usize).into());
    0 as i32
}

#[no_mangle]
pub extern "C" fn shyper_yield() {
    crate::libs::thread::thread_yield()
}

fn nanoseconds_to_timespec(nanoseconds: usize, result: &mut Timespec) {
    result.tv_sec = (nanoseconds / 1_000_000_000) as i64;
    result.tv_nsec = (nanoseconds % 1_000_000_000) as i64;
}

#[no_mangle]
pub extern "C" fn shyper_clock_gettime(clock_id: u64, tp: *mut Timespec) -> i32 {
    use crate::libs::timer::{CLOCK_REALTIME, CLOCK_MONOTONIC, current_ns, boot_time};
    assert!(
        !tp.is_null(),
        "shyper_clock_gettime called with a zero tp parameter"
    );
    let result = unsafe { &mut *tp };
    match clock_id {
        CLOCK_REALTIME | CLOCK_MONOTONIC => {
            let mut nanoseconds = current_ns();

            if clock_id == CLOCK_REALTIME {
                nanoseconds += boot_time() * 1000;
            }

            nanoseconds_to_timespec(nanoseconds, result);
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
    // #[cfg(feature = "net")]
    // crate::libs::net::network_init();
    // Currently we do nothing here.
    0
}

#[cfg(feature = "unwind")]
static mut GLOBAL_PAYLOAD: u64 = 0;

/// Store payload addr during std panic unwind process.
/// Todo: it's not thread safe.
#[cfg(feature = "unwind")]
pub(crate) fn get_global_payload() -> u64 {
    unsafe {
        debug!("get_global_payload, payload {:x}", GLOBAL_PAYLOAD);
        GLOBAL_PAYLOAD
    }
}

#[no_mangle]
pub fn shyper_start_panic(payload: *mut u8) -> ! {
    debug!(
        "shyper_start_panic, payload {:p} {:x}",
        payload, payload as u64
    );
    #[cfg(feature = "unwind")]
    {
        unsafe {
            GLOBAL_PAYLOAD = payload as u64;
        }
        crate::libs::unwind::unwind_from_panic(9)
    }

    #[cfg(not(feature = "unwind"))]
    {
        shyper_abort();
        loop {}
    }
}
