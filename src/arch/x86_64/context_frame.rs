use core::fmt::Formatter;

use crate::libs::traits::ContextFrameTrait;

use super::mpk::pkru_of_thread_id;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct X86_64ContextFrame {
    gpr: GeneralRegs,
}

/// General registers
#[repr(C, align(16))]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct GeneralRegs {
    /// PKRU, for Intel MPK support.
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

impl core::fmt::Display for X86_64ContextFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
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

impl ContextFrameTrait for X86_64ContextFrame {
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
                    "X86_64ContextFrame get register value of invalid index {}",
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
                    "X86_64ContextFrame set register to value {:#x} of invalid index {}",
                    value, index
                );
            }
        }
    }

    fn set_from_irq(&mut self) {}

    fn set_from_yield(&mut self) {}
}

macro_rules! save_context {
    () => {
        concat!(
            r#"
            pushfq
			push rax
			push rcx
			push rdx
			push rbx
			push rbp
			push rsi
			push rdi
			push r8
			push r9
			push r10
			push r11
			push r12
			push r13
			push r14
			push r15
            rdfsbase rax
		    push rax
			"#
        ) // Qemu CPU fsgsbase feature is enabled.
    };
}

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

macro_rules! restore_context {
    () => {
        concat!(
            // Qemu CPU fsgsbase feature is enabled.
            r#"
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
			ret
			"#
        )
    };
}

use core::arch::asm;

/// The entry of `thread_yield` operation,
/// It will trigger the scheduler to actively yield to next thread,
/// which contains the following logic:
/// 1.  save the registers on current thread's stack space;
/// 2.  call `switch_to_next_stack`, which will call the `core.schedule()` to pick the next thread,
///         and switch current stack pointer to next thread.
/// 3.  when `switch_to_next_stack` returns, the `rax` holds the next thread's stack pointer,
///         we can get if the thread is yield from irq or yield,
///         from irq: jump to pop_context_first.
///         from yield, just set up the necessary registers and br 'x30'.
#[naked]
pub extern "C" fn yield_to() {
    unsafe {
        asm!(
            save_context!(),
            save_pkru!(),
            // Switch pkru to kernel mode.
            "xor ecx, ecx",
            "xor edx, edx",
            "xor rax, rax",
            "wrpkru",
            // Pass current stack pointer,
            "mov rdi, rsp",
            "call {switch_to_next_stack}",
            "mov rsp, rax",
            // Set task switched flag, CR0 bit 3
            "mov rax, cr0",
            "or rax, 8",
            "mov cr0, rax",
            restore_pkru!(),
            restore_context!(),
            switch_to_next_stack = sym switch_to_next_stack,
            options(noreturn)
        );
    }
}

use super::interface::ContextFrame;
#[no_mangle]
extern "C" fn switch_to_next_stack(ctx: *mut ContextFrame) -> usize {
    // if ctx as usize != 0 {
    //     unsafe {
    //         println!("yield to ctx on user_sp {:p}\n {}", ctx, ctx.read());
    //     }
    // }

    // Store current context's pointer on current core struct.
    // Note: ctx is just a pointer to current thread stack.
    let core = crate::libs::cpu::cpu();

    // debug!(
    //     "switch_to_next_stack is called on thread [{}] current pkru {:#x}",
    //     core.running_thread().unwrap().tid(),
    //     super::mpk::rdpkru()
    // );

    core.set_current_sp(ctx as usize);

    core.schedule();

    core.current_sp()
}

pub fn set_thread_id(_tid: u64) {}

pub fn get_tls_ptr() -> *const u8 {
    0xDEAD_BEEF as *const u8
}

pub fn set_tls_ptr(_tls_ptr: u64) {}

pub unsafe extern "C" fn pop_context_first(ctx: usize) -> ! {
    debug!("get pkru {:#x}", super::mpk::rdpkru());

    super::mpk::wrpkru(super::mpk::pkru_of_zone_id(1));

    debug!("get modified pkru {:#x}", super::mpk::rdpkru());

    _pop_context_first(ctx);
    loop {}
}

#[naked]
unsafe extern "C" fn _pop_context_first(_next_stack: usize) {
    // `next_stack` is in `rdi` register
    asm!(
        "cli",
        "mov rsp, rdi",
        restore_pkru!(),
        restore_context!(),
        options(noreturn),
    )
}
