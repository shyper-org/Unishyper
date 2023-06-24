use core::fmt::Formatter;
use core::arch::asm;

use crate::libs::traits::ContextFrameTrait;

#[cfg(feature = "mpk")]
use super::mpk::pkru_of_thread_id;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct X86_64TrapContextFrame {
    gpr: GeneralRegs,
}

/// General registers
#[repr(C, align(16))]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct GeneralRegs {
    /// PKRU, for Intel MPK support.
    #[cfg(feature = "mpk")]
    pub pkru: usize,
    /// FS register for TLS support.
    pub fsbase: usize,
    /// R15 register, callee saved.
    pub r15: usize,
    /// R14 register, callee saved.
    pub r14: usize,
    /// R13 register, callee saved.
    pub r13: usize,
    /// R12 register, callee saved.
    pub r12: usize,
    /// R11 register, caller saved.
    pub r11: usize,
    /// R10 register, caller saved.
    pub r10: usize,
    /// Sixth argument, caller saved.
    pub r9: usize,
    /// Fifth argument, caller saved.
    pub r8: usize,
    // First argument, callee saved.
    pub rdi: usize,
    // Second argument, callee saved.
    pub rsi: usize,
    /// RBP register, callee saved.
    pub rbp: usize,
    /// RBX register, callee saved.
    pub rbx: usize,
    /// Third argument, caller saved.
    pub rdx: usize,
    /// Fourth argument, caller saved.
    pub rcx: usize,
    /// Function return value, caller saved.
    pub rax: usize,
    /// Status Flags.
    pub rflags: usize,
    /// Instruction Pointer Register.
    pub rip: usize,
    /// Stack pointer, callee saved.
    pub rsp: usize,
}

impl core::fmt::Display for X86_64TrapContextFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        #[cfg(feature = "mpk")]
        write!(f, "pkru: {:016x} ", self.gpr.pkru)?;
        writeln!(f, "fsbase:{:016x}", self.gpr.fsbase)?;
        write!(f, "r15: {:016x} ", self.gpr.r15)?;
        writeln!(f, "r14: {:016x} ", self.gpr.r14)?;
        write!(f, "r13: {:016x} ", self.gpr.r13)?;
        writeln!(f, "r12: {:016x} ", self.gpr.r12)?;
        write!(f, "r11: {:016x} ", self.gpr.r11)?;
        writeln!(f, "r10: {:016x} ", self.gpr.r10)?;
        write!(f, " r9: {:016x} ", self.gpr.r9)?;
        writeln!(f, " r8: {:016x} ", self.gpr.r8)?;
        write!(f, "rdi: {:016x} ", self.gpr.rdi)?;
        writeln!(f, "rsi: {:016x} ", self.gpr.rsi)?;
        write!(f, "rbp: {:016x} ", self.gpr.rbp)?;
        writeln!(f, "rbx: {:016x} ", self.gpr.rbx)?;
        write!(f, "rdx: {:016x} ", self.gpr.rdx)?;
        writeln!(f, "rcx: {:016x} ", self.gpr.rcx)?;
        write!(f, "rax: {:016x} ", self.gpr.rax)?;
        writeln!(f, "rflags: {:016x} ", self.gpr.rflags)?;
        write!(f, "rip: {:016x} ", self.gpr.rip)?;
        writeln!(f, "rsp: {:016x} ", self.gpr.rsp)?;
        Ok(())
    }
}

impl ContextFrameTrait for X86_64TrapContextFrame {
    #[cfg(not(feature = "mpk"))]
    fn init(&mut self, _tid: usize) {
        self.gpr.rflags = 0x1202;
    }

    #[cfg(feature = "mpk")]
    fn init(&mut self, tid: usize) {
        self.gpr.rflags = 0x1202;
        self.gpr.pkru = pkru_of_thread_id(tid) as usize;
    }

    fn exception_pc(&self) -> usize {
        self.gpr.rip
    }

    fn set_exception_pc(&mut self, pc: usize) {
        self.gpr.rip = pc;
    }

    fn stack_pointer(&self) -> usize {
        self.gpr.rsp
    }

    fn set_stack_pointer(&mut self, sp: usize) {
        self.gpr.rsp = sp;
    }

    fn gpr(&self, index: usize) -> usize {
        match index {
            0 => self.gpr.rdi,
            1 => self.gpr.rsi,
            2 => self.gpr.rdx,
            3 => self.gpr.rcx,
            4 => self.gpr.r8,
            5 => self.gpr.r9,
            _ => {
                warn!(
                    "X86_64TrapContextFrame get register value of invalid index {}",
                    index
                );
                0
            }
        }
    }

    fn set_gpr(&mut self, index: usize, value: usize) {
        match index {
            0 => self.gpr.rdi = value,
            1 => self.gpr.rsi = value,
            2 => self.gpr.rdx = value,
            3 => self.gpr.rcx = value,
            4 => self.gpr.r8 = value,
            5 => self.gpr.r9 = value,
            _ => {
                warn!(
                    "X86_64TrapContextFrame set register to value {:#x} of invalid index {}",
                    value, index
                );
            }
        }
    }
}

/// Callee-saved registers
#[repr(C)]
#[derive(Debug, Default)]
struct YieldContextFrame {
    #[cfg(feature = "mpk")]
    pkru: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    rip: u64,
}

pub struct ThreadContext {
    rsp: u64,
}

impl ThreadContext {
    /// Creates a new default `ThreadContext` for a new thread.
    pub const fn new() -> Self {
        Self { rsp: 0 }
    }

    /// Switches to another thread to its yield context.
    /// The yield context means callee-saved registers.
    /// It first saves the current thread's context from CPU to this place, and then
    /// restores the next thread's callee-saved registers from `next_ctx` to CPU.
    pub fn switch_to_yield_ctx(&mut self, next_ctx: &Self) {
        // debug!(
        //     "switch_to_yield_ctx prev rsp {:#x}, next rsp {:#x}",
        //     self.rsp, next_ctx.rsp
        // );
        unsafe { context_switch_to_yield(&mut self.rsp, next_ctx.rsp) }
    }

    /// Switches to another thread to its trap context.
    /// The trap context means the whole trap context frame.
    /// It first saves the current thread's context from CPU to this place, and then
    /// restores the next thread's trap context from `next_sp` to CPU.
    pub fn switch_to_trap_ctx(&mut self, next_sp: usize) {
        // debug!(
        //     "switch_to_trap_ctx prev rsp {:#x}, next rsp {:#x}",
        //     self.rsp, next_sp
        // );
        unsafe { context_switch_to_trap(&mut self.rsp, next_sp) }
    }
}

/// Save prev context (callee-saved registers) into current stack, see `YieldContextFrame` for details.
macro_rules! save_yield_context {
    () => {
        concat!(
            r#"
            push rbp
            push rbx
            push r12
            push r13
            push r14
            push r15
			"#
        )
    };
}

/// Pop next context (callee-saved registers) from current stack, see `YieldContextFrame` for details.
macro_rules! restore_yield_context {
    () => {
        concat!(
            r#"
            pop r15
            pop r14
            pop r13
            pop r12
            pop rbx
            pop rbp
			"#
        )
    };
}

/// Pop next context (whole context frame) from current stack, see `X86_64TrapContextFrame` for details.
macro_rules! restore_trap_context {
    () => {
        concat!(
            r#"
            // restore new context
            pop rax
            wrfsbase rax
			pop r15
			pop r14
			pop r13
			pop r12
			pop r11
			pop r10
			pop r9
			pop r8
			pop rdi
			pop rsi
			pop rbp
			pop rbx
			pop rdx
			pop rcx
			pop rax
			popfq
			"#
        )
    };
}

/// Save pkru register into current stack.
#[cfg(feature = "mpk")]
macro_rules! save_pkru {
    () => {
        concat!(
            r#"
            xor rax, rax
            xor ecx, ecx
            rdpkru
            push rax
            "#
        )
    };
}

/// Restore pkru register from current stack.
#[cfg(feature = "mpk")]
macro_rules! restore_pkru {
    () => {
        concat!(
            r#"
            pop rax
            xor ecx, ecx
            xor edx, edx
            wrpkru
            "#
        )
    };
}

/// The actual process of `thread_yield` operation,
/// It will trigger the scheduler to actively yield to next thread, which is runned before.
///
/// Which means that next thread's thread context is stored as `YieldContextFrame` in `_next_stack`.
///
/// Context switch process to a newly allocated thread, see `context_switch_to_trap`.
/// ## Arguments
/// * `_current_stack`  - the pointer to prev stack pointer(rsp), on `rdi`.
/// * `_next_stack`     - next stack pointer(rsp), on `rsi`.
#[cfg(not(feature = "mpk"))]
#[naked]
unsafe extern "C" fn context_switch_to_yield(_current_stack: &mut u64, _next_stack: u64) {
    asm!(
        save_yield_context!(),
        // Switch stack pointer.
        "mov    [rdi], rsp",
        "mov    rsp, rsi",
        // Set task switched flag, CR0 bit 3
        // "mov rax, cr0",
        // "or rax, 8",
        // "mov cr0, rax",
        restore_yield_context!(),
        "ret",
        options(noreturn),
    )
}

#[cfg(feature = "mpk")]
#[naked]
unsafe extern "C" fn context_switch_to_yield(_current_stack: &mut u64, _next_stack: u64) {
    asm!(
        save_yield_context!(),
        save_pkru!(),
        // Switch pkru to kernel mode.
        "xor ecx, ecx",
        "xor edx, edx",
        "xor rax, rax",
        "wrpkru",
        // Switch stack pointer.
        "mov    [rdi], rsp",
        "mov    rsp, rsi",
        // Set task switched flag, CR0 bit 3
        "mov rax, cr0",
        "or rax, 8",
        "mov cr0, rax",
        restore_pkru!(),
        restore_yield_context!(),
        "ret",
        options(noreturn),
    )
}

/// The actual process of `thread_yield` operation,
/// It will trigger the scheduler to actively yield to next thread, which is not runned before.
///
/// Which means that next thread's thread context is stored as `X86_64TrapContextFrame` in `_next_sp`.
///
/// Context switch process to a runned thread, see `context_switch_to_yield`
/// ## Arguments
/// * `_current_stack`  - the pointer to prev stack pointer(rsp), on `rdi`.
/// * `_next_sp`     - next stack pointer(rsp), on `rsi`.
#[cfg(not(feature = "mpk"))]
#[naked]
unsafe extern "C" fn context_switch_to_trap(_current_stack: &mut u64, _next_sp: usize) {
    asm!(
        save_yield_context!(),
        // Switch stack pointer.
        "mov    [rdi], rsp",
        "mov    rsp, rsi",
        // Set task switched flag, CR0 bit 3
        "mov rax, cr0",
        "or rax, 8",
        "mov cr0, rax",
        restore_trap_context!(),
        "ret",
        options(noreturn),
    )
}

#[cfg(feature = "mpk")]
#[naked]
unsafe extern "C" fn context_switch_to_trap(_current_stack: &mut u64, _next_sp: usize) {
    asm!(
        save_yield_context!(),
        save_pkru!(),
        // Switch pkru to kernel mode.
        "xor ecx, ecx",
        "xor edx, edx",
        "xor rax, rax",
        "wrpkru",
        // Switch stack pointer.
        "mov    [rdi], rsp",
        "mov    rsp, rsi",
        // Set task switched flag, CR0 bit 3
        "mov rax, cr0",
        "or rax, 8",
        "mov cr0, rax",
        restore_pkru!(),
        restore_trap_context!(),
        "ret",
        options(noreturn),
    )
}

/// Pop first thread's context frame and jump to it.
/// Called by `pop_context_first`.
#[cfg(not(feature = "mpk"))]
#[naked]
pub(super) unsafe extern "C" fn _pop_context_first(_next_stack: usize) {
    // `_next_stack` is in `rdi` register
    asm!(
        "cli",
        "mov rsp, rdi",
        restore_trap_context!(),
        "ret",
        options(noreturn),
    )
}
#[cfg(feature = "mpk")]
#[naked]
pub(super) unsafe extern "C" fn _pop_context_first(_next_stack: usize) {
    // `_next_stack` is in `rdi` register
    asm!(
        "cli",
        "mov rsp, rdi",
        restore_pkru!(),
        restore_trap_context!(),
        "ret",
        options(noreturn),
    )
}
