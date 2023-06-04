use core::fmt::Formatter;
use core::arch::asm;

use riscv::regs::SSTATUS;

use crate::libs::traits::ContextFrameTrait;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Riscv64TrapContextFrame {
    gpr: [u64; 32],
    sstatus: u64,
    sepc: u64,
}

static REG_ABI_NAMES: [&str; 32] = [
    "ZERO", "RA", "SP", "GP", "TP", "T0", "T1", "T2", "S0/FP", "S1", "A0", "A1", "A2", "A3", "A4",
    "A5", "A6", "A7", "S2", "S3", "S4", "S5", "S6", "S7", "S8", "S9", "S10", "S11", "T3", "T4",
    "T5", "T6",
];

impl core::fmt::Display for Riscv64TrapContextFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        for i in 0..32 {
            write!(f, "{:5}: {:016x}   ", REG_ABI_NAMES[i], self.gpr[i])?;
            if (i + 1) % 2 == 0 {
                write!(f, "\n")?;
            }
        }
        write!(f, "{:5}: {:016x}   ", "SSTAT", self.sstatus)?;
        writeln!(f, "{:5}: {:016x}   ", "EPC", self.sepc)?;
        Ok(())
    }
}

impl ContextFrameTrait for Riscv64TrapContextFrame {
    fn init(&mut self, _tid: usize) {
        self.sstatus = (SSTATUS::SD::SET
            + SSTATUS::FS.val(0b11)
            // The SUM (permit Supervisor User Memory access) bit
            // modifies the privilege with which S-mode loads and stores access virtual memory.
            // When SUM=0, S-mode memory accesses to pages that are accessible by U-mode (U=1 in Figure 4.18) will fault.
            // When SUM=1, these accesses are permitted.
            + SSTATUS::SUM::SET
            // The SPP bit indicates the privilege level at which a hart was executing before entering supervisor mode. 
            // When a trap is taken, SPP is set to 0 if the trap originated from user mode, or 1 otherwise.
            // When an SRET instruction (see Section 3.2.2) is executed to return from the trap handler, 
            // the privilege level is set to user mode if the SPP bit is 0
            // or supervisor mode if the SPP bit is 1; SPP is then set to 0.
            + SSTATUS::SPP::Supervisor
            // The SPIE bit indicates whether supervisor interrupts were enabled prior to trapping into supervisor mode.
            // When a trap is taken into supervisor mode, SPIE is set to SIE, and SIE is set to 0. 
            // When an SRET instruction is executed, SIE is set to SPIE, then SPIE is set to 1.
            + SSTATUS::SPIE.val(1)
            // The SIE bit enables or disables all interrupts in supervisor mode. 
            // When SIE is clear, interrupts are not taken while in supervisor mode. 
            // When the hart is running in user-mode, the value in SIE isignored, and supervisor-level interrupts are enabled. 
            // The supervisor can disable indivdual interrupt sources using the sie register.
            + SSTATUS::SIE.val(0))
        .value;
    }
    fn exception_pc(&self) -> usize {
        self.sepc as usize
    }

    fn set_exception_pc(&mut self, pc: usize) {
        self.sepc = pc as u64;
    }
    fn stack_pointer(&self) -> usize {
        // sp -> x2
        self.gpr[2] as usize
    }
    fn set_stack_pointer(&mut self, sp: usize) {
        self.gpr[2] = sp as u64;
    }
    fn gpr(&self, index: usize) -> usize {
        self.gpr[index + 10] as usize
    }
    fn set_gpr(&mut self, index: usize, value: usize) {
        self.gpr[index + 10] = value as u64;
    }
}

/// Saved hardware states of a task.
///
/// The context usually includes:
///
/// - Callee-saved registers
/// - Stack pointer register
/// - Thread pointer register (for thread-local storage, currently unsupported)
/// - FP/SIMD registers
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
#[repr(C)]
#[derive(Debug, Default)]
pub struct ThreadContext {
    pub ra: usize, // return address (x1)
    pub sp: usize, // stack pointer (x2)

    pub s0: usize, // x8-x9
    pub s1: usize,

    pub s2: usize, // x18-x27
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
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

/// Save prev context (callee-saved registers) into heap space `a0`, see `ThreadContext` for details.
macro_rules! save_yield_context {
    () => {
        concat!(
            r#"
            sd  ra, 0*8(a0)
            sd  sp, 1*8(a0)
            sd  s0, 2*8(a0)
            sd  s1, 3*8(a0)
            sd  s2, 4*8(a0)
            sd  s3, 5*8(a0)
            sd  s4, 6*8(a0)
            sd  s5, 7*8(a0)
            sd  s6, 8*8(a0)
            sd  s7, 9*8(a0)
            sd  s8, 10*8(a0)
            sd  s9, 11*8(a0)
            sd  s10, 12*8(a0)
            sd  s11, 13*8(a0)
			"#
        )
    };
}

/// Pop new context (callee-saved registers) from heap space `a1`, see `ThreadContext` for details.
macro_rules! restore_yield_context {
    () => {
        concat!(
            r#"
            ld  s11, 13*8(a1)
            ld  s10, 12*8(a1)
            ld  s9, 11*8(a1)
            ld  s8, 10*8(a1)
            ld  s7, 9*8(a1)
            ld  s6, 8*8(a1)
            ld  s5, 7*8(a1)
            ld  s4, 6*8(a1)
            ld  s3, 5*8(a1)
            ld  s2, 4*8(a1)
            ld  s1, 3*8(a1)
            ld  s0, 2*8(a1)
            ld  sp, 1*8(a1)
            ld  ra, 0*8(a1)
			"#
        )
    };
}

/// Pop next context (whole context frame) from `x1`, see `TrapContextFrame` for details.
macro_rules! restore_trap_context {
    () => {
        concat!(
            r#"
            mv sp, a1
            
            ld s1, 32 * 8(sp)
            ld s2, 33 * 8(sp)
            csrw sstatus, s1
            csrw sepc, s2

            ld x1, 1 * 8(sp)
            // no x2(sp) here
            ld x3, 3 * 8(sp)
            ld x4, 4 * 8(sp)
            ld x5, 5 * 8(sp)
            ld x6, 6 * 8(sp)
            ld x7, 7 * 8(sp)
            ld x8, 8 * 8(sp)
            ld x9, 9 * 8(sp)
            ld x10, 10 * 8(sp)
            ld x11, 11 * 8(sp)
            ld x12, 12 * 8(sp)
            ld x13, 13 * 8(sp)
            ld x14, 14 * 8(sp)
            ld x15, 15 * 8(sp)
            ld x16, 16 * 8(sp)
            ld x17, 17 * 8(sp)
            ld x18, 18 * 8(sp)
            ld x19, 19 * 8(sp)
            ld x20, 20 * 8(sp)
            ld x21, 21 * 8(sp)
            ld x22, 22 * 8(sp)
            ld x23, 23 * 8(sp)
            ld x24, 24 * 8(sp)
            ld x25, 25 * 8(sp)
            ld x26, 26 * 8(sp)
            ld x27, 27 * 8(sp)
            ld x28, 28 * 8(sp)
            ld x29, 29 * 8(sp)
            ld x30, 30 * 8(sp)
            ld x31, 31 * 8(sp)

            ld x2, 2 * 8(sp)// restore user sp
            sret
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
