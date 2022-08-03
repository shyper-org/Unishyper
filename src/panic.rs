use core::panic::PanicInfo;

use crate::lib::thread::current_thread;

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(message) = info.message() {
        if current_thread().is_ok() {
            error!(
                "PANIC on Thread [{}]: {}",
                current_thread().unwrap().tid(),
                message
            );
        }
    }
    if let Some(location) = info.location() {
        error!("Location: {}:{}", location.file(), location.line());
    }
    loop {}
}
