use core::fmt::Formatter;
use core::arch::asm;

use super::registers::Aarch64;
use super::registers::Registers;

use crate::libs::traits::ContextFrameTrait;

/// AArch64 state (Exception level and selected SP) that an exception was taken from. The
/// possible values are:
///
/// M[3:0] | State
/// --------------
/// 0b0000 | EL0t
/// 0b0100 | EL1t
/// 0b0101 | EL1h
/// 0b1000 | EL2t
/// 0b1001 | EL2h
///
/// Other values are reserved, and returning to an Exception level that is using AArch64
/// with a reserved value in this field is treated as an illegal exception return.
///
/// The bits in this field are interpreted as follows:
///   - M[3:2] holds the Exception Level.
///   - M[1] is unused and is RES 0 for all non-reserved values.
///   - M[0] is used to select the SP:
///     - 0 means the SP is always SP0.
///     - 1 means the exception SP is determined by the EL.

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct TrapContextFrame {
    /// General purpose registers, x0 to x30.
    gpr: [u64; 31],
    /// By default, spsr is set to 0x44, which is 0b0100_0100 in binray.
    /// (SPSR_EL1::M::EL1t + SPSR_EL1::I::Unmasked + SPSR_EL1::F::Masked).value as u64
    spsr: u64, // 31 * 8
    /// Exception return address.
    /// During initialization, ELR_EL1 is set to the thread entry address,
    /// which is "thread_start".
    elr: u64, // 32 * 8
    /// Stack pointer.
    sp: u64, // 33 * 8
    /// It's a mark showing the thread is yield from irq or thread_yield.
    /// These two conditions may result in different context restore processes.
    /// 1. from irq: pop_context_first
    /// 2. from yield: see switch.S for details.
    from_interrupt: bool, // 34 * 8
}

/// Saved hardware states of a task.
///
/// The context usually includes:
///
/// - Callee-saved registers
/// - Stack pointer register
/// - Thread pointer register (for thread-local storage, currently unsupported)
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
#[repr(C)]
#[derive(Debug)]
pub struct ThreadContext {
    pub sp: u64,
    pub tpidr_el0: u64,
    pub r19: u64,
    pub r20: u64,
    pub r21: u64,
    pub r22: u64,
    pub r23: u64,
    pub r24: u64,
    pub r25: u64,
    pub r26: u64,
    pub r27: u64,
    pub r28: u64,
    pub r29: u64,
    pub lr: u64, // r30
}

impl Into<Registers> for TrapContextFrame {
    fn into(self) -> Registers {
        let mut reg = Registers::default();
        reg[Aarch64::X0] = Some(self.gpr[0]);
        reg[Aarch64::X1] = Some(self.gpr[1]);
        reg[Aarch64::X2] = Some(self.gpr[2]);
        reg[Aarch64::X3] = Some(self.gpr[3]);
        reg[Aarch64::X4] = Some(self.gpr[4]);
        reg[Aarch64::X5] = Some(self.gpr[5]);
        reg[Aarch64::X6] = Some(self.gpr[6]);
        reg[Aarch64::X7] = Some(self.gpr[7]);
        reg[Aarch64::X8] = Some(self.gpr[8]);
        reg[Aarch64::X9] = Some(self.gpr[9]);
        reg[Aarch64::X10] = Some(self.gpr[10]);
        reg[Aarch64::X11] = Some(self.gpr[11]);
        reg[Aarch64::X12] = Some(self.gpr[12]);
        reg[Aarch64::X13] = Some(self.gpr[13]);
        reg[Aarch64::X14] = Some(self.gpr[14]);
        reg[Aarch64::X15] = Some(self.gpr[15]);
        reg[Aarch64::X16] = Some(self.gpr[16]);
        reg[Aarch64::X17] = Some(self.gpr[17]);
        reg[Aarch64::X18] = Some(self.gpr[18]);
        reg[Aarch64::X19] = Some(self.gpr[19]);
        reg[Aarch64::X20] = Some(self.gpr[20]);
        reg[Aarch64::X21] = Some(self.gpr[21]);
        reg[Aarch64::X22] = Some(self.gpr[22]);
        reg[Aarch64::X23] = Some(self.gpr[23]);
        reg[Aarch64::X24] = Some(self.gpr[24]);
        reg[Aarch64::X25] = Some(self.gpr[25]);
        reg[Aarch64::X26] = Some(self.gpr[26]);
        reg[Aarch64::X27] = Some(self.gpr[27]);
        reg[Aarch64::X28] = Some(self.gpr[28]);
        reg[Aarch64::X29] = Some(self.gpr[29]);
        reg[Aarch64::X30] = Some(self.gpr[30]);
        reg[Aarch64::SP] = Some(self.sp);
        reg
    }
}

impl core::fmt::Display for TrapContextFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        for i in 0..31 {
            write!(f, "x{:02}: {:016x}   ", i, self.gpr[i])?;
            if (i + 1) % 2 == 0 {
                write!(f, "\n")?;
            }
        }
        writeln!(f, "spsr:{:016x}", self.spsr)?;
        write!(f, "elr: {:016x}", self.elr)?;
        writeln!(f, "   sp:  {:016x}", self.sp)?;
        writeln!(
            f,
            "this thread recently yield from '{}'",
            if self.from_interrupt {
                "interrupt"
            } else {
                "thread_yield"
            }
        )?;
        Ok(())
    }
}

impl ContextFrameTrait for TrapContextFrame {
    fn init(&mut self, _tid: usize) {}

    fn exception_pc(&self) -> usize {
        self.elr as usize
    }

    fn set_exception_pc(&mut self, pc: usize) {
        self.elr = pc as u64;
    }

    fn stack_pointer(&self) -> usize {
        self.sp as usize
    }

    fn set_stack_pointer(&mut self, sp: usize) {
        self.sp = sp as u64;
    }

    fn gpr(&self, index: usize) -> usize {
        assert!(index < crate::arch::registers::GPR_NUM_MAX);
        self.gpr[index] as usize
    }

    fn set_gpr(&mut self, index: usize, value: usize) {
        assert!(index < crate::arch::registers::GPR_NUM_MAX);
        self.gpr[index] = value as u64;
    }
    #[cfg(feature = "zone")]
    fn set_pkru(&mut self, _value: u32) {}

    #[cfg(feature = "zone")]
    fn pkru(&self) -> u32 {
        0
    }
}

impl ThreadContext {
    /// Creates a new default `ThreadContext` for a new thread.
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    /// Switches to another thread to its yield context.
    /// The yield context means callee-saved registers.
    /// It first saves the current thread's context from CPU to this place, and then
    /// restores the next thread's callee-saved registers from `next_ctx` to CPU.
    pub fn switch_to_yield_ctx(&mut self, next_ctx: &Self) {
        unsafe { context_switch_to_yield(self, next_ctx) }
    }

    /// Switches to another thread to its trap context.
    /// The trap context means the whole trap context frame.
    /// It first saves the current thread's context from CPU to this place, and then
    /// restores the next thread's trap context from `next_sp` to CPU.
    pub fn switch_to_trap_ctx(&mut self, next_sp: usize) {
        unsafe { context_switch_to_trap(self, next_sp) }
    }
}

/// Save prev context (callee-saved registers) into heap space `x0`, see `ThreadContext` for details.
macro_rules! save_yield_context {
    () => {
        concat!(
            r#"
            stp     x29, x30, [x0, 12 * 8]
            stp     x27, x28, [x0, 10 * 8]
            stp     x25, x26, [x0, 8 * 8]
            stp     x23, x24, [x0, 6 * 8]
            stp     x21, x22, [x0, 4 * 8]
            stp     x19, x20, [x0, 2 * 8]
            mov     x19, sp
            mrs     x20, tpidr_el0
            stp     x19, x20, [x0]
			"#
        )
    };
}

/// Pop new context (callee-saved registers) from heap space `x1`, see `ThreadContext` for details.
macro_rules! restore_yield_context {
    () => {
        concat!(
            r#"
            ldp     x19, x20, [x1]
            mov     sp, x19
            msr     tpidr_el0, x20
            ldp     x19, x20, [x1, 2 * 8]
            ldp     x21, x22, [x1, 4 * 8]
            ldp     x23, x24, [x1, 6 * 8]
            ldp     x25, x26, [x1, 8 * 8]
            ldp     x27, x28, [x1, 10 * 8]
            ldp     x29, x30, [x1, 12 * 8]
			"#
        )
    };
}

/// Pop next context (whole context frame) from `x1`, see `TrapContextFrame` for details.
macro_rules! restore_trap_context {
    () => {
        concat!(
            r#"
            // restore new context
            mov sp, x1
            mov x0, #0x45
            ldr x1, [sp, #(32 * 8)] // elr
            ldr x2, [sp, #(33 * 8)] // sp
            msr spsr_el1, x0
            msr elr_el1, x1
            msr sp_el0, x2
            ldp x0, x1,  [sp, #(0 * 16)]
            ldp x2, x3,  [sp, #(1 * 16)]
            ldp x4, x5,  [sp, #(2 * 16)]
            ldp x6, x7,  [sp, #(3 * 16)]
            ldp x8, x9,  [sp, #(4 * 16)]
            ldp x10,x11, [sp, #(5 * 16)]
            ldp x12,x13, [sp, #(6 * 16)]
            ldp x14,x15, [sp, #(7 * 16)]
            ldp x16,x17, [sp, #(8 * 16)]
            ldp x18,x19, [sp, #(9 * 16)]
            ldp x20,x21, [sp, #(10 * 16)]
            ldp x22,x23, [sp, #(11 * 16)]
            ldp x24,x25, [sp, #(12 * 16)]
            ldp x26,x27, [sp, #(13 * 16)]
            ldp x28,x29, [sp, #(14 * 16)]
            ldr x30, [sp, #(15 * 16)]
            add	sp, sp, 0x120
            eret
			"#
        )
    };
}

/// The actual process of `thread_yield` operation,
/// It will trigger the scheduler to actively yield to next thread, which is runned before.
///
/// Which means that next thread's thread context is stored as `ThreadContext` in `_next_ctx`.
///
/// Context switch process to a newly allocated thread, see `context_switch_to_trap`.
/// ## Arguments
/// * `_current_ctx`  - the pointer to prev thread's `ThreadContext`.
/// * `_next_ctx`     - the pointer to next thread's `ThreadContext`.
#[naked]
unsafe extern "C" fn context_switch_to_yield(
    _current_ctx: &mut ThreadContext,
    _next_ctx: &ThreadContext,
) {
    asm!(
        save_yield_context!(),
        restore_yield_context!(),
        "ret",
        options(noreturn),
    )
}

/// The actual process of `thread_yield` operation,
/// It will trigger the scheduler to actively yield to next thread, which is not runned before.
///
/// Which means that next thread's thread context is stored as `TrapContextFrame` in `_next_sp`.
///
/// Context switch process to a runned thread, see `context_switch_to_yield`
/// ## Arguments
/// * `_current_ctx`    - the pointer to prev thread's `ThreadContext`.
/// * `_next_sp`        - next stack pointer(rsp), on `x1`.
#[naked]
unsafe extern "C" fn context_switch_to_trap(_current_ctx: &mut ThreadContext, _next_sp: usize) {
    asm!(
        save_yield_context!(),
        restore_trap_context!(),
        options(noreturn),
    )
}
