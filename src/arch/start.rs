core::arch::global_asm!(include_str!("start.S"));

#[no_mangle]
pub fn _start() {
    extern "C" {
        fn __start() -> !;
    }
    unsafe {
        __start();
    }
}