core::arch::global_asm!(include_str!("start.S"));

extern "C" {
    fn _start() -> !;
}
