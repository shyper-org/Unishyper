use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
  if let Some(message) = info.message() {
    error!("PANIC: {}", message);
  }
  if let Some(location) = info.location() {
    error!("Location: {}:{}", location.file(), location.line());
  }
  loop {}
}