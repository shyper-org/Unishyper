use tock_registers::*;
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::*;

use crate::board::{GICC_BASE, GICD_BASE};
use crate::libs::traits::InterruptControllerTrait;
use crate::libs::traits::ArchTrait;

const GICR_BASE: usize = 0xfe680000;
const GIC_INTERRUPT_NUM: usize = 1024;
const GIC_PPI_NUM: usize = 16;
const GIC_SGI_NUM: usize = 16;
const GIC_PRIVINT_NUM: usize = GIC_SGI_NUM + GIC_PPI_NUM;

const GIC_INT_REGS_NUM: usize = GIC_INTERRUPT_NUM / 32;
const GIC_PRIO_REGS_NUM: usize = GIC_INTERRUPT_NUM * 8 / 32;
const GIC_TARGET_REGS_NUM: usize = GIC_INTERRUPT_NUM * 8 / 32;
const GIC_CONFIG_REGS_NUM: usize = GIC_INTERRUPT_NUM * 2 / 32;
const GIC_SEC_REGS_NUM: usize = GIC_INTERRUPT_NUM * 2 / 32;
const GIC_SGI_REGS_NUM: usize = GIC_SGI_NUM * 8 / 32;
const MPIDR_AFF_MSK: usize = 0xffff; //we are only supporting 2 affinity levels
const GICD_IROUTER_INV: usize = !MPIDR_AFF_MSK;
const GICR_WAKER_PSLEEP_BIT: usize = 0x2;
const GICR_WAKER_CASLEEP_BIT: usize = 0x4;
const GICD_IROUTER_RES0_MSK: usize = (1 << 40) - 1;
pub const GICD_IROUTER_IRM_BIT: usize = 1 << 31;
const GICD_IROUTER_AFF_MSK: usize = GICD_IROUTER_RES0_MSK & !GICD_IROUTER_IRM_BIT;

register_structs! {
    #[allow(non_snake_case)]
    pub GicDistributorBlock {
        (0x0000 => CTLR: ReadWrite<u32>), //Distributor Control Register
        (0x0004 => TYPER: ReadOnly<u32>), //Interrupt Controller Type Register
        (0x0008 => IIDR: ReadOnly<u32>),  //Distributor Implementer Identification Register
        (0x000c => TYPER2: ReadOnly<u32>), //Interrupt controller Type Register 2
        (0x0010 => STATUSR: ReadWrite<u32>), //Error Reporting Status Register, optional
        (0x0014 => reserved0),
        (0x0040 => SETSPI_NSR: WriteOnly<u32>), //Set SPI Register
        (0x0044 => reserved1),
        (0x0048 => CLRSPI_NSR: WriteOnly<u32>), //Clear SPI Register
        (0x004c => reserved2),
        (0x0050 => SETSPI_SR: WriteOnly<u32>), //Set SPI, Secure Register
        (0x0054 => reserved3),
        (0x0058 => CLRSPI_SR: WriteOnly<u32>), //Clear SPI, Secure Register
        (0x005c => reserved4),
        (0x0080 => IGROUPR: [ReadWrite<u32>; GIC_INT_REGS_NUM]), //Interrupt Group Registers
        (0x0100 => ISENABLER: [ReadWrite<u32>; GIC_INT_REGS_NUM]), //Interrupt Set-Enable Registers
        (0x0180 => ICENABLER: [ReadWrite<u32>; GIC_INT_REGS_NUM]), //Interrupt Clear-Enable Registers
        (0x0200 => ISPENDR: [ReadWrite<u32>; GIC_INT_REGS_NUM]), //Interrupt Set-Pending Registers
        (0x0280 => ICPENDR: [ReadWrite<u32>; GIC_INT_REGS_NUM]), //Interrupt Clear-Pending Registers
        (0x0300 => ISACTIVER: [ReadWrite<u32>; GIC_INT_REGS_NUM]), //Interrupt Set-Active Registers
        (0x0380 => ICACTIVER: [ReadWrite<u32>; GIC_INT_REGS_NUM]), //Interrupt Clear-Active Registers
        (0x0400 => IPRIORITYR: [ReadWrite<u32>; GIC_PRIO_REGS_NUM]), //Interrupt Priority Registers
        (0x0800 => ITARGETSR: [ReadWrite<u32>; GIC_TARGET_REGS_NUM]), //Interrupt Processor Targets Registers
        (0x0c00 => ICFGR: [ReadWrite<u32>; GIC_CONFIG_REGS_NUM]), //Interrupt Configuration Registers
        (0x0d00 => IGRPMODR: [ReadWrite<u32>; GIC_CONFIG_REGS_NUM]), //Interrupt Group Modifier Registers
        (0x0e00 => NSACR: [ReadWrite<u32>; GIC_SEC_REGS_NUM]), //Non-secure Access Control Registers
        (0x0f00 => SGIR: WriteOnly<u32>),  //Software Generated Interrupt Register
        (0x0f04 => reserved6),
        (0x0f10 => CPENDSGIR: [ReadWrite<u32>; GIC_SGI_REGS_NUM]), //SGI Clear-Pending Registers
        (0x0f20 => SPENDSGIR: [ReadWrite<u32>; GIC_SGI_REGS_NUM]), //SGI Set-Pending Registers
        (0x0f30 => reserved7),
        (0x6000 => IROUTER: [ReadWrite<u64>; (0x8000 - 0x6000) / core::mem::size_of::<u64>()]), //Interrupt Routing Registers for extended SPI range
        (0x8000 => reserved21),
        (0xffd0 => ID: [ReadOnly<u32>;(0x10000 - 0xffd0) / core::mem::size_of::<u32>()]), //Reserved for ID registers
        (0x10000 => @END),
    }
}

struct GicDistributor {
    base_addr: usize,
}

impl core::ops::Deref for GicDistributor {
    type Target = GicDistributorBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

register_structs! {
  #[allow(non_snake_case)]
  GicCpuInterfaceBlock {
    (0x0000 => CTLR: ReadWrite<u32>),   // CPU Interface Control Register
    (0x0004 => PMR: ReadWrite<u32>),    // Interrupt Priority Mask Register
    (0x0008 => BPR: ReadWrite<u32>),    // Binary Point Register
    (0x000c => IAR: ReadOnly<u32>),     // Interrupt Acknowledge Register
    (0x0010 => EOIR: WriteOnly<u32>),   // End of Interrupt Register
    (0x0014 => RPR: ReadOnly<u32>),     // Running Priority Register
    (0x0018 => HPPIR: ReadOnly<u32>),   // Highest Priority Pending Interrupt Register
    (0x001c => ABPR: ReadWrite<u32>),   // Aliased Binary Point Register
    (0x0020 => AIAR: ReadOnly<u32>),    // Aliased Interrupt Acknowledge Register
    (0x0024 => AEOIR: WriteOnly<u32>),  // Aliased End of Interrupt Register
    (0x0028 => AHPPIR: ReadOnly<u32>),  // Aliased Highest Priority Pending Interrupt Register
    (0x002c => _reserved_0),
    (0x00d0 => APR: [ReadWrite<u32>; 4]),    // Active Priorities Register
    (0x00e0 => NSAPR: [ReadWrite<u32>; 4]),  // Non-secure Active Priorities Register
    (0x00f0 => _reserved_1),
    (0x00fc => IIDR: ReadOnly<u32>),    // CPU Interface Identification Register
    (0x0100 => _reserved_2),
    (0x1000 => DIR: WriteOnly<u32>),    // Deactivate Interrupt Register
    (0x1004 => _reserved_3),
    (0x2000 => @END),
  }
}

struct GicCpuInterface {
    base_addr: usize,
}

impl core::ops::Deref for GicCpuInterface {
    type Target = GicCpuInterfaceBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl GicCpuInterface {
    const fn new(base_addr: usize) -> Self {
        GicCpuInterface { base_addr }
    }

    fn ptr(&self) -> *const GicCpuInterfaceBlock {
        self.base_addr as *const _
    }

    fn init(&self) {
        let pmr = 0xffu64;
        let bpr1 = 0x0u64;
        let igroupr0 = 1u64;
        unsafe {
            core::arch::asm!("msr ICC_PMR_EL1, {}", in(reg) pmr);
            core::arch::asm!("msr ICC_BPR1_EL1, {}", in(reg) bpr1);
            core::arch::asm!("msr ICC_IGRPEN1_EL1, {}", in(reg) igroupr0);
        }
    }
}

impl GicDistributor {
    const fn new(base_addr: usize) -> Self {
        GicDistributor { base_addr }
    }

    fn ptr(&self) -> *const GicDistributorBlock {
        self.base_addr as *const _
    }

    fn init(&self) {
        let max_spi = (self.TYPER.get() & 0b11111 + 1) * 32;
        for i in 1usize..(max_spi as usize / 32) {
            self.IGROUPR[i].set(u32::MAX);
            self.ICENABLER[i].set(u32::MAX);
            self.ICPENDR[i].set(u32::MAX);
            self.ICACTIVER[i].set(u32::MAX);
        }
        for i in 8usize..(max_spi as usize * 8 / 32) {
            self.IPRIORITYR[i].set(u32::MAX);
        }

        for i in GIC_PRIVINT_NUM..GIC_INTERRUPT_NUM {
            self.IROUTER[i].set(GICD_IROUTER_INV as u64);
        }
        self.CTLR.set(self.CTLR.get() | 0x10 | 0b10);
    }

    fn set_enable(&self, int: usize) {
        let idx = int / 32;
        let bit = 1u32 << (int % 32);
        self.ISENABLER[idx].set(bit);
    }

    fn clear_enable(&self, int: usize) {
        let idx = int / 32;
        let bit = 1u32 << (int % 32);
        self.ICENABLER[idx].set(bit);
    }

    fn set_priority(&self, int: usize, priority: u8) {
        let idx = (int * 8) / 32;
        let offset = (int * 8) % 32;
        let mask: u32 = 0b11111111 << offset;
        let prev = self.IPRIORITYR[idx].get();
        self.IPRIORITYR[idx].set((prev & (!mask)) | (((priority as u32) << offset) & mask));
    }

    fn set_config(&self, int: usize, edge: bool) {
        let idx = (int * 2) / 32;
        let offset = (int * 2) % 32;
        let mask: u32 = 0b11 << offset;
        let prev = self.ICFGR[idx].get();
        self.ICFGR[idx].set((prev & (!mask)) | ((if edge { 0b10 } else { 0b00 } << offset) & mask));
    }

    pub fn set_route(&self, int_id: usize, route: usize) {
        self.IROUTER[int_id as usize].set((route & GICD_IROUTER_AFF_MSK) as u64);
    }
}

register_structs! {
    #[allow(non_snake_case)]
    pub GicRedistributorBlock {
        (0x0000 => CTLR: ReadWrite<u32>),   // Redistributor Control Register
        (0x0004 => IIDR: ReadOnly<u32>),    // Implementer Identification Register
        (0x0008 => TYPER: ReadOnly<u64>),   // Redistributor Type Register
        (0x0010 => STATUSR: ReadWrite<u32>),  // Error Reporting Status Register, optional
        (0x0014 => WAKER: ReadWrite<u32>),     // Redistributor Wake Register
        (0x0018 => MPAMIDR: ReadOnly<u32>),   // Report maximum PARTID and PMG Register
        (0x001c => PARTIDR: ReadWrite<u32>),   // Set PARTID and PMG Register
        (0x0020 => reserved18),
        (0x0040 => SETLPIR: WriteOnly<u64>),    // Set LPI Pending Register
        (0x0048 => CLRLPIR: WriteOnly<u64>),  // Clear LPI Pending Register
        (0x0050 => reserved17),
        (0x0070 => PROPBASER: ReadWrite<u64>),  //Redistributor Properties Base Address Register
        (0x0078 => PEDNBASER: ReadWrite<u64>),    //Redistributor LPI Pending Table Base Address Register
        (0x0080 => reserved16),
        (0x00a0 => INVLPIR: WriteOnly<u64>),  // Redistributor Invalidate LPI Register
        (0x00a8 => reserved15),
        (0x00b0 => INVALLR: WriteOnly<u64>),    // Redistributor Invalidate All Register
        (0x00b8 => reserved14),
        (0x00c0 => SYNCR: ReadOnly<u64>),    // Redistributor Synchronize Register
        (0x00c8 => reserved13),
        (0xffd0 => ID: [ReadOnly<u32>;(0x10000 - 0xFFD0) / 4]),
        (0x10000 => reserved12),
        (0x10080 => IGROUPR0: ReadWrite<u32>), //SGI_base frame, all below
        (0x10084 => reserved11),
        (0x10100 => ISENABLER0: ReadWrite<u32>),
        (0x10104 => reserved10),
        (0x10180 => ICENABLER0: ReadWrite<u32>),
        (0x10184 => reserved9),
        (0x10200 => ISPENDR0: ReadWrite<u32>),
        (0x10204 => reserved8),
        (0x10280 => ICPENDR0: ReadWrite<u32>),
        (0x10284 => reserved7),
        (0x10300 => ISACTIVER0: ReadWrite<u32>),
        (0x10304 => reserved6),
        (0x10380 => ICACTIVER0: ReadWrite<u32>),
        (0x10384 => reserved5),
        (0x10400 => IPRIORITYR: [ReadWrite<u32>;8]),
        (0x10420 => reserved4),
        (0x10c00 => ICFGR0: ReadWrite<u32>),
        (0x10c04 => ICFGR1: ReadWrite<u32>),
        (0x10c08 => reserved3),
        (0x10d00 => IGRPMODR0: ReadWrite<u32>),
        (0x10d04 => reserved2),
        (0x10e00 => NSACR: ReadWrite<u32>),
        (0x10e04 => reserved1),
        (0x20000 => @END),
  }
}

pub struct GicRedistributor {
    base_addr: usize,
}

impl core::ops::Deref for GicRedistributor {
    type Target = GicRedistributorBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl core::ops::Index<usize> for GicRedistributor {
    type Output = GicRedistributorBlock;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            &*((self.ptr() as usize + index * core::mem::size_of::<GicRedistributorBlock>())
                as *const GicRedistributorBlock)
        }
    }
}

impl GicRedistributor {
    pub const fn new(base_addr: usize) -> GicRedistributor {
        GicRedistributor { base_addr }
    }

    pub fn ptr(&self) -> *const GicRedistributorBlock {
        self.base_addr as *const GicRedistributorBlock
    }

    fn init(&self, cpu_id: usize) {
        let waker = self[cpu_id].WAKER.get();
        self[cpu_id]
            .WAKER
            .set(waker & !GICR_WAKER_PSLEEP_BIT as u32);
        while (self[cpu_id].WAKER.get() & GICR_WAKER_CASLEEP_BIT as u32) != 0 {}

        self[cpu_id].IGROUPR0.set(u32::MAX);
        self[cpu_id].ICENABLER0.set(u32::MAX);
        self[cpu_id].ICPENDR0.set(u32::MAX);
        self[cpu_id].ICACTIVER0.set(u32::MAX);

        for i in 0..(GIC_PRIVINT_NUM * 8 / 32) as usize {
            self[cpu_id].IPRIORITYR[i].set(u32::MAX);
        }
    }

    fn set_priority(&self, int_id: usize, prio: u8, gicr_id: u32) {
        let reg_id = int_id * 8 / 32;
        let off = (int_id * 8) % 32;
        let mask = (((1 << ((8) - 1)) << 1) - 1) << (off);

        self[gicr_id as usize].IPRIORITYR[reg_id].set(
            (self[gicr_id as usize].IPRIORITYR[reg_id].get() & !mask as u32)
                | (((prio as usize) << off) & mask) as u32,
        );
    }

    // pub fn set_config(&self, int_id: usize, cfg: u8, gicr_id: u32) {
    //     let reg_id = (int_id * 2) / 32;
    //     let off = (int_id * 2) % 32;
    //     let mask = ((1 << (2)) - 1) << (off);

    //     match reg_id {
    //         0 => {
    //             self[gicr_id as usize].ICFGR0.set(
    //                 ((self[gicr_id as usize].ICFGR0.get() as usize & !mask)
    //                     | (((cfg as usize) << off) & mask)) as u32,
    //             );
    //         }
    //         _ => {
    //             self[gicr_id as usize].ICFGR1.set(
    //                 ((self[gicr_id as usize].ICFGR1.get() as usize & !mask)
    //                     | (((cfg as usize) << off) & mask)) as u32,
    //             );
    //         }
    //     }
    // }

    pub fn set_enable(&self, int_id: usize, gicr_id: u32) {
        let mask = 1 << (int_id % 32);
        self[gicr_id as usize].ISENABLER0.set(mask);
    }

    pub fn clear_enable(&self, int_id: usize, gicr_id: u32) {
        let mask = 1 << (int_id % 32);
        self[gicr_id as usize].ICENABLER0.set(mask);
    }
}

static GICD: GicDistributor = GicDistributor::new(GICD_BASE | 0xFFFF_FF80_0000_0000);
static GICC: GicCpuInterface = GicCpuInterface::new(GICC_BASE | 0xFFFF_FF80_0000_0000);
pub static GICR: GicRedistributor = GicRedistributor::new(GICR_BASE | 0xFFFF_FF80_0000_0000);

pub struct InterruptController;

impl InterruptControllerTrait for InterruptController {
    fn init() {
        let core_id = crate::arch::Arch::core_id();
        let gicd = &GICD;
        let gicr = &GICR;
        if core_id == 0 {
            gicd.init();
        }
        crate::util::barrier();
        let gicc = &GICC;
        gicr.init(core_id);
        gicc.init();
    }

    fn enable(int: Interrupt) {
        let core_id = crate::arch::Arch::core_id();
        debug!("core {} gic enable interrupt {}", core_id, int);
        if (int as usize) < 32 {
            let gicr = &GICR;
            gicr.set_enable(int, core_id as u32);
            gicr.set_priority(int, 0x7f, core_id as u32);
        } else {
            let gicd = &GICD;
            gicd.set_enable(int);
            gicd.set_priority(int, 0x7f);
            gicd.set_config(int, true);
            gicd.set_route(int, core_id << 8);
        }
    }

    fn disable(int: Interrupt) {
        if (int as usize) < 32 {
            let gicr = &GICR;
            gicr.clear_enable(int, crate::arch::Arch::core_id() as u32);
        } else {
            let gicd = &GICD;
            gicd.clear_enable(int);
        }
    }

    fn fetch() -> Option<Interrupt> {
        let i: usize;
        unsafe {
            core::arch::asm!("mrs {}, ICC_IAR1_EL1", out(reg) i);
        }
        if i >= 1022 {
            None
        } else {
            Some(i as Interrupt)
        }
    }

    fn finish(int: Interrupt) {
        unsafe {
            core::arch::asm!("msr ICC_EOIR1_EL1,{}", in(reg) int);
        }
    }
}

pub const INT_TIMER: Interrupt = 27; // virtual timer

pub type Interrupt = usize;
