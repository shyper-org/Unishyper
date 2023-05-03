use core::fmt::Formatter;

use cortex_a::registers::{TPIDRRO_EL0, TPIDR_EL0};
use tock_registers::interfaces::{Writeable, Readable};

use super::registers::Aarch64;
use super::registers::Registers;

use crate::libs::traits::ContextFrameTrait;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct Aarch64ContextFrame {
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

impl Into<Registers> for Aarch64ContextFrame {
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

impl core::fmt::Display for Aarch64ContextFrame {
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

impl ContextFrameTrait for Aarch64ContextFrame {
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

    fn set_from_irq(&mut self) {
        self.from_interrupt = true;
    }

    fn set_from_yield(&mut self) {
        self.from_interrupt = false;
    }
}

/// The entry of `thread_yield` operation,
/// It will trigger the scheduler to actively yield to next thread,
/// which contains the following logic:
/// 1.  jump to `save_context_on_current_stack` to save the registers on current thread's stack space;
/// 2.  call `switch_to_next_stack`, which will call the `core.schedule()` to pick the next thread,
///         and switch current stack pointer to next thread.
/// 3.  when `switch_to_next_stack` returns, the x0 holds the next thread's stack pointer,
///         we can get if the thread is yield from irq or yield,
///         from irq: jump to pop_context_first.
///         from yield, just set up the necessary registers and br 'x30'.
#[inline(always)]
pub fn yield_to() {
    extern "C" {
        fn save_context_on_current_stack();
    }

    unsafe {
        save_context_on_current_stack();
    }
    // Enable interrupt after return to this thread.
}

use super::interface::ContextFrame;
#[no_mangle]
extern "C" fn switch_to_next_stack(ctx: *mut ContextFrame) -> usize {
    // use tock_registers::interfaces::Readable;
    // use cortex_a::registers::TPIDRRO_EL0;
    // debug!(
    //     "yield_to called on thread [{}], ctx on user_sp {:p}\n",
    //     TPIDRRO_EL0.get(),
    //     ctx,
    // );
    // if ctx as usize != 0 {
    //     println!("{}", ctx.read());
    // }

    // Store current context's pointer on current core struct.
    // Note: ctx is just a pointer to current thread stack.
    let core = crate::libs::cpu::cpu();

    core.set_current_sp(ctx as usize);

    core.schedule();

    core.current_sp()
}

pub fn set_thread_id(tid: u64) {
    TPIDRRO_EL0.set(tid);
}

pub fn get_tls_ptr() -> *const u8 {
    TPIDR_EL0.get() as *const u8
}

pub fn set_tls_ptr(tls_ptr: u64) {
    TPIDR_EL0.set(tls_ptr);
}

#[inline(always)]
pub unsafe extern "C" fn pop_context_first(ctx: usize) -> ! {
    extern "C" {
        fn _pop_context_first(ctx: usize) -> !;
    }
    _pop_context_first(ctx)
}
