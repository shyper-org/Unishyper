use riscv::regs::SSTATUS;
use tock_registers::interfaces::{Readable, ReadWriteable};

/// Enable Interrupts
#[inline]
pub fn enable() {
    SSTATUS.modify(SSTATUS::SIE::SET)
    // SSTATUS_SET.write(SSTATUS::SIE.val(1))
}

/// Enable Interrupts and wait for the next interrupt (HLT instruction)
/// According to <https://lists.freebsd.org/pipermail/freebsd-current/2004-June/029369.html>, this exact sequence of assembly
/// instructions is guaranteed to be atomic.
/// This is important, because another CPU could call wakeup_core right when we decide to wait for the next interrupt.
#[allow(unused)]
#[inline]
pub fn enable_and_wait() {
    enable();
    riscv::asm::wfi()
}

/// Disable Interrupts
// #[inline]
#[inline(never)]
#[no_mangle]
pub fn disable() {
    SSTATUS.modify(SSTATUS::SIE::CLEAR);
}

/// Disable IRQs (nested)
///
/// Disable IRQs when unsure if IRQs were enabled at all.
/// This function together with nested_enable can be used
/// in situations when interrupts shouldn't be activated if they
/// were not activated before calling this function.
#[inline]
pub fn nested_disable() -> bool {
    let ret = SSTATUS.is_set(SSTATUS::SIE);
    disable();
    ret
}

/// Enable IRQs (nested)
///
/// Can be used in conjunction with nested_disable() to only enable
/// interrupts again if they were enabled before.
#[inline]
pub fn nested_enable(was_enabled: bool) {
    if was_enabled {
        enable();
    }
}
