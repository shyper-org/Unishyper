use riscv::regs::*;
use tock_registers::interfaces::{Readable, Writeable, ReadWriteable};

use crate::arch::ContextFrame;
use crate::libs::interrupt::InterruptController;
use crate::libs::traits::*;

core::arch::global_asm!(include_str!("exception.S"));

// Interrupt Exception_Code Description
//      1        1          Supervisor software interrupt
//      1        5          Supervisor timer interrupt
//      1        9          Supervisor external interrupt
//      1        ≥16        Designated for platform use
//      0        0          Instruction address misaligned
//      0        1          Instruction access fault
//      0        2          Illegal instruction
//      0        3          Breakpoint
//      0        4          Load address misaligned
//      0        5          Load access fault
//      0        6          Store/AMO address misaligned
//      0        7          Store/AMO access fault
//      0        8          Environment call from U-mode
//      0        9          Environment call from S-mode
//      0        10–11      Reserved
//      0        12         Instruction page fault
//      0        13         Load page fault
//      0        15         Store/AMO page fault
//      0        16–23      Reserved
//      0        24–31      Designated for custom use
//      0        48–63      Designated for custom use

const INTERRUPT_SUPERVISOR_SOFTWARE: usize = 1;
const INTERRUPT_SUPERVISOR_TIMER: usize = 5;
const INTERRUPT_SUPERVISOR_EXTERNAL: usize = 9;

#[no_mangle]
unsafe extern "C" fn exception_entry(ctx: *mut ContextFrame) {
    // Supervisor Cause Register.
    let cause = SCAUSE.get();
    // Bit 63 holds the Interrupt bit.
    // The Interrupt bit in the scause register is set if the trap was caused by an interrupt.
    let irq = (cause >> 63) != 0;
    // The Exception Code field contains a code identifying the last exception or interrupt
    let code = (cause & 0xf) as usize;

    // debug!(
    //     "exception_entry, irq {}, code {}\n ctx on sp {:p}\n",
    //     irq, code, ctx,
    // );
    // println!("{}", ctx.read());

    if irq {
        match code {
            INTERRUPT_SUPERVISOR_SOFTWARE => panic!("Interrupt::SupervisorSoft"),
            INTERRUPT_SUPERVISOR_TIMER => {
                crate::libs::timer::interrupt();
                crate::libs::thread::thread_yield();
            }
            INTERRUPT_SUPERVISOR_EXTERNAL => {
                let plic = &crate::drivers::INTERRUPT_CONTROLLER;
                if let Some(int) = plic.fetch() {
                    crate::libs::interrupt::interrupt(int);
                    plic.finish(int);
                } else {
                    warn!("PLIC report no irq");
                }
            }
            _ => panic!("Interrupt::Unknown"),
        }
    } else {
        warn!("SCAUSE {:016x}", cause);
        warn!("SEPC {:016x}", ctx.read().exception_pc());
        warn!("FAR  {:016x}", crate::arch::Arch::fault_address());
        panic!("Unhandled kernel exception");
    }
}

pub fn init() {
    extern "C" {
        fn push_context();
    }
    STVEC.write(STVEC::BASE.val(push_context as usize as u64 >> 2) + STVEC::MODE::Direct);
    // Note: riscv vector only 4 byte per cause
    //       direct mode make it distributed later in `exception_entry`
    SIE.modify(SIE::SEIE::SET);

    // The SUM (permit Supervisor User Memory access) bit
    // modifies the privilege with which S-mode loads and stores access virtual memory.
    // When SUM=0, S-mode memory accesses to pages that are accessible by U-mode (U=1 in Figure 4.18) will fault.
    // When SUM=1, these accesses are permitted.
    SSTATUS.write(SSTATUS::SUM::SET);
}
