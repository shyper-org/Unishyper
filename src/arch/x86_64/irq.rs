use core::arch::asm;
use x86_64::registers::rflags::{self, RFlags};

/// Enable Interrupts
#[inline]
pub fn enable() {
    unsafe {
        asm!("sti");
    }
}

/// Enable Interrupts and wait for the next interrupt (HLT instruction)
/// According to <https://lists.freebsd.org/pipermail/freebsd-current/2004-June/029369.html>, this exact sequence of assembly
/// instructions is guaranteed to be atomic.
/// This is important, because another CPU could call wakeup_core right when we decide to wait for the next interrupt.
#[allow(unused)]
#[inline]
pub fn enable_and_wait() {
    unsafe {
        asm!("sti; hlt", options(nomem, nostack));
    }
}

/// Disable Interrupts
#[inline]
pub fn disable() {
    unsafe {
        asm!("cli");
    }
}

/// Disable IRQs (nested)
///
/// Disable IRQs when unsure if IRQs were enabled at all.
/// This function together with nested_enable can be used
/// in situations when interrupts shouldn't be activated if they
/// were not activated before calling this function.
#[inline]
pub fn nested_disable() -> bool {
    let ret = rflags::read().contains(RFlags::INTERRUPT_FLAG);
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
