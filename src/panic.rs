#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn _Unwind_Resume(arg: usize) -> ! {
    info!("Unwind resume arg {}", arg);
    loop {}
}

#[cfg(not(feature = "std"))]
#[panic_handler]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // use crate::libs::thread::current_thread;
    // if let Some(message) = info.message() {
    //     if current_thread().is_ok() {
    //         error!(
    //             "PANIC on Thread [{}]: {}",
    //             current_thread().unwrap().tid(),
    //             message
    //         );
    //     } else {
    //         error!(
    //             "PANIC on  None Thread : {}",
    //             message
    //         );
    //     }
    // }
    // use crate::libs::thread::current_thread;
    if let Some(message) = info.message() {
        error!("PANIC on : {}", message);
    }
    if let Some(location) = info.location() {
        error!("Location: {}:{}", location.file(), location.line());
    }

    // #[cfg(feature = "unwind")]
    // crate::libs::unwind::unwind_from_panic(3);
    // #[cfg(not(feature = "unwind"))]
    loop {}
}
