use core::panic::PanicInfo;

use crate::lib::thread::current_thread;

#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn _Unwind_Resume(arg: usize) -> ! {
    info!("Unwind resume arg {}",arg);
    loop {}
}

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
