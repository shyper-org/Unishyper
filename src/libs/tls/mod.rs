/// Simple thread local key implementation.
/// Refer to implementation in https://github.com/rust-lang/rust/tree/master/library/std/src/sys/sgx/abi/tls
mod sync_bitset;

use self::sync_bitset::*;

use core::cell::Cell;
use core::mem;
use core::num::NonZeroUsize;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(target_pointer_width = "64")]
const USIZE_BITS: usize = 64;
const TLS_KEYS: usize = 128; // Same as POSIX minimum
const TLS_KEYS_BITSET_SIZE: usize = (TLS_KEYS + (USIZE_BITS - 1)) / USIZE_BITS;

static TLS_KEY_IN_USE: SyncBitset = SYNC_BITSET_INIT;

static TLS_DESTRUCTOR: [AtomicUsize; TLS_KEYS] = [const { AtomicUsize::new(0) }; TLS_KEYS];

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Key(NonZeroUsize);

impl Key {
    fn to_index(self) -> usize {
        self.0.get() - 1
    }

    fn from_index(index: usize) -> Self {
        Key(NonZeroUsize::new(index + 1).unwrap())
    }

    pub fn as_usize(self) -> usize {
        self.0.get()
    }

    pub fn from_usize(index: usize) -> Self {
        Key(NonZeroUsize::new(index).unwrap())
    }
}

#[repr(C)]
pub struct Tls {
    data: [Cell<*mut u8>; TLS_KEYS],
    // data: [Rc<Cell<*mut u8>>; TLS_KEYS],
}

pub struct ActiveTls<'a> {
    tls: &'a Tls,
}

impl<'a> Drop for ActiveTls<'a> {
    fn drop(&mut self) {
        debug!("ActiveTls drop ,self at {:p}", &self.tls.data);
        let value_with_destructor = |key: usize| {
            let ptr = TLS_DESTRUCTOR[key].load(Ordering::Relaxed);
            unsafe { mem::transmute::<_, Option<unsafe extern "C" fn(*mut u8)>>(ptr) }
                .map(|dtor| (&self.tls.data[key], dtor))
        };

        let mut any_non_null_dtor = true;
        while any_non_null_dtor {
            any_non_null_dtor = false;
            for (value, dtor) in TLS_KEY_IN_USE.iter().filter_map(&value_with_destructor) {
                // let value = value.replace(ptr::null_mut());
                let value = value.replace(ptr::null_mut());
                if !value.is_null() {
                    any_non_null_dtor = true;
                    unsafe { dtor(value) }
                }
            }
        }
    }
}

use crate::arch::PAGE_SIZE;
use crate::mm::paging::MappedRegion;
use crate::mm::address::VAddr;
use crate::util::round_up;

#[derive(Debug)]
pub struct ThreadTls {
    region: MappedRegion,
}

impl ThreadTls {
    pub fn get_tls_start(&self) -> VAddr {
        self.region.start_address()
    }
}

impl Drop for ThreadTls {
    fn drop(&mut self) {
        debug!("ThreadTls drop , start at {}", self.get_tls_start());
        let self_tls = unsafe { &*(self.get_tls_start().as_ptr::<u8>() as *const Tls) };
        let value_with_destructor = |key: usize| {
            let ptr = TLS_DESTRUCTOR[key].load(Ordering::Relaxed);
            unsafe { mem::transmute::<_, Option<unsafe extern "C" fn(*mut u8)>>(ptr) }
                .map(|dtor| (&self_tls.data[key], dtor))
        };

        let mut any_non_null_dtor = true;
        while any_non_null_dtor {
            any_non_null_dtor = false;
            for (value, dtor) in TLS_KEY_IN_USE.iter().filter_map(&value_with_destructor) {
                let value = value.replace(ptr::null_mut());
                if !value.is_null() {
                    any_non_null_dtor = true;
                    unsafe { dtor(value) }
                }
            }
        }
    }
}

pub fn alloc_thread_local_storage_region(zone_id: crate::libs::zone::ZoneId) -> ThreadTls {
    let tls_size = round_up(core::mem::size_of::<Tls>(), PAGE_SIZE);
    let region = crate::mm::allocate_region(tls_size, Some(zone_id))
        .expect("failed to alloc region for tls");
    ThreadTls { region }
}

impl Tls {
    pub fn new() -> Tls {
        let tls = Tls {
            data: [const { Cell::new(ptr::null_mut()) }; TLS_KEYS],
        };
        debug!("tls alloc at {:p}", &tls);
        return tls;
    }

    unsafe fn current<'a>() -> &'a Tls {
        use crate::libs::traits::ArchTrait;
        unsafe { &*(crate::arch::Arch::get_tls_ptr() as *const Tls) }
    }

    pub fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
        let index = if let Some(index) = TLS_KEY_IN_USE.set() {
            index
        } else {
            panic!("TLS limit exceeded")
        };
        TLS_DESTRUCTOR[index].store(dtor.map_or(0, |f| f as usize), Ordering::Relaxed);
        unsafe { Self::current() }.data[index].set(ptr::null_mut());
        // let key = Key::from_index(index);
        // debug!(
        //     "tls key create, index {}, key: {:?} usize {}",
        //     index,
        //     key,
        //     key.as_usize()
        // );
        Key::from_index(index)
    }

    pub fn set(key: Key, value: *mut u8) {
        let index = key.to_index();
        assert!(TLS_KEY_IN_USE.get(index));

        // debug!(
        //     "tls key set to value {:x} , index {}, key: {:?} usize {}",
        //     value as usize,
        //     index,
        //     key,
        //     key.as_usize()
        // );
        unsafe { Self::current() }.data[index].set(value);
    }

    pub fn get(key: Key) -> *mut u8 {
        let index = key.to_index();
        assert!(TLS_KEY_IN_USE.get(index));
        // let value = unsafe { Self::current() }.data[index].get();
        // debug!(
        //     "tls key get the value {:x} , index {}, key: {:?} usize {}",
        //     value as usize,
        //     index,
        //     key,
        //     key.as_usize()
        // );
        unsafe { Self::current() }.data[index].get()
    }

    pub fn destroy(key: Key) {
        // debug!(
        //     "tls key destroy, index {}, key: {:?} usize {}",
        //     key.to_index(),
        //     key,
        //     key.as_usize()
        // );
        TLS_KEY_IN_USE.clear(key.to_index());
    }
}
