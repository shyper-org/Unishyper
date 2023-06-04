/// Shyper unikernel abi for tls operations.
/// See src/libs/tls for more details.
use crate::libs::tls::{Key as TlsKey, Tls};

pub type Key = usize;

#[no_mangle]
pub extern "C" fn shyper_thread_local_key_create(
    dtor: Option<unsafe extern "C" fn(*mut u8)>,
) -> Key {
    Tls::create(dtor).as_usize()
}

#[no_mangle]
pub extern "C" fn shyper_thread_local_key_set(key: Key, value: *mut u8) {
    Tls::set(TlsKey::from_usize(key), value)
}

#[no_mangle]
pub extern "C" fn shyper_thread_local_key_get(key: Key) -> *mut u8 {
    Tls::get(TlsKey::from_usize(key))
}

#[no_mangle]
pub extern "C" fn shyper_thread_local_key_destroy(key: Key) {
    Tls::destroy(TlsKey::from_usize(key))
}
