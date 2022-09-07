use core::ops::Range;

use cortex_a::registers::*;
use tock_registers::interfaces::Readable;

pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0x8000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;

use crate::board::BOARD_CORE_NUMBER;
use crate::lib::traits::*;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

#[allow(unused)]
pub const KERNEL_STACK_SIZE: usize = 32_768; // // PAGE_SIZE * 8

pub const STACK_SIZE: usize = 1_048_576; // PAGE_SIZE * 256

const PA2KVA: usize = 0xFFFF_FF80_0000_0000;
const KVA2PA: usize = 0x0000_007F_FFFF_FFFF;

impl Address for usize {
    fn pa2kva(&self) -> usize {
        *self | PA2KVA
    }
    fn kva2pa(&self) -> usize {
        *self & KVA2PA
    }
}

pub type ContextFrame = super::context_frame::Aarch64ContextFrame;

pub type CoreId = usize;

pub struct Arch;

impl ArchTrait for Arch {
    fn exception_init() {
        super::exception::init();
    }

    fn invalidate_tlb() {
        unsafe {
            core::arch::asm!("dsb ishst");
            core::arch::asm!("tlbi vmalle1is");
            core::arch::asm!("dsb ish");
            core::arch::asm!("isb");
        }
    }

    fn wait_for_interrupt() {
        cortex_a::asm::wfi();
    }

    fn nop() {
        cortex_a::asm::nop();
    }

    fn fault_address() -> usize {
        FAR_EL1.get() as usize
    }

    fn core_id() -> CoreId {
        MPIDR_EL1.get() as usize & (BOARD_CORE_NUMBER - 1)
    }

    fn curent_privilege() -> usize {
        (CurrentEL.get() as usize & 0b1100) >> 2
    }
}
