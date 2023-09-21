// use apic::{LocalApic, XApic, LAPIC_ADDR};
use x2apic::ioapic::IoApic;
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder};
use x86_64::instructions::port::Port;
use spin::Once;

use crate::libs::synch::spinlock::SpinlockIrqSave;
use crate::libs::traits::{ArchTrait, InterruptControllerTrait};

// use self::vectors::*;

// pub(super) mod vectors {
// pub const APIC_TIMER_VECTOR: u8 = 0xf0;
// pub const APIC_SPURIOUS_VECTOR: u8 = 0xf1;
// pub const APIC_ERROR_VECTOR: u8 = 0xf2;
pub const TIMER_INTERRUPT_NUMBER: u8 = 123;
pub const ERROR_INTERRUPT_NUMBER: u8 = 126;
pub const SPURIOUS_INTERRUPT_NUMBER: u8 = 127;
// }

/// The maximum number of IRQs.
// pub const MAX_IRQ_COUNT: usize = 256;

pub const IRQ_MIN: usize = 0x20;

/// The timer IRQ number.
pub const INT_TIMER: usize = TIMER_INTERRUPT_NUMBER as usize;

const IO_APIC_BASE: usize = 0xFEC0_0000;

static mut LOCAL_APIC: Option<LocalApic> = None;
static mut IS_X2APIC: bool = false;
static mut IO_APIC: Once<SpinlockIrqSave<IoApic>> = Once::new();

pub struct InterruptController;

#[allow(unused)]
impl InterruptControllerTrait for InterruptController {
    fn init() {
        if crate::arch::Arch::core_id() != 0 {
            unsafe { local_apic().enable() };
        } else {
            info!("Initialize Local APIC...");
            unsafe {
                // Disable 8259A interrupt controllers
                Port::<u8>::new(0x21).write(0xff);
                Port::<u8>::new(0xA1).write(0xff);
            }

            let mut builder = LocalApicBuilder::new();
            builder
                .timer_vector(TIMER_INTERRUPT_NUMBER as _)
                .error_vector(ERROR_INTERRUPT_NUMBER as _)
                .spurious_vector(SPURIOUS_INTERRUPT_NUMBER as _);

            let mut is_x2apic = false;

            if cpu_has_x2apic() {
                info!("Using x2APIC.");
                is_x2apic = true;
            } else {
                // let base_vaddr = (unsafe { xapic_base() } as usize).pa2kva();
                let xapic_base_paddr = unsafe { xapic_base() } as usize;
                let xapic_base_vaddr = crate::mm::paging::map_device_memory_range(
                    xapic_base_paddr,
                    crate::arch::PAGE_SIZE,
                );
                info!(
                    "Using xAPIC. paddr at {:#x} map to {:?}",
                    xapic_base_paddr, xapic_base_vaddr
                );
                builder.set_xapic_base(xapic_base_vaddr.value() as u64);
            }

            let mut lapic = builder.build().unwrap();
            unsafe {
                lapic.enable();
                LOCAL_APIC = Some(lapic);
            }

            info!("Initialize IO APIC...");
            let ioapic_paddr = IO_APIC_BASE as usize;
            let ioapic_vaddr =
                crate::mm::paging::map_device_memory_range(ioapic_paddr, crate::arch::PAGE_SIZE);

            info!(
                "Initialize IO APIC paddr at {:#x} map to {:?}",
                ioapic_paddr, ioapic_vaddr
            );

            unsafe {
                let mut io_apic = IoApic::new(ioapic_vaddr.value() as u64);
                io_apic.init(0x21);

                let max_entry = io_apic.max_table_entry() + 1;
                info!(
                    "IOAPIC id {} v{} has {} entries",
                    io_apic.id(),
                    io_apic.version(),
                    max_entry
                );
                for i in 0..max_entry {
                    if i != 2 {
                        io_apic.enable_irq(i);
                    } else {
                        io_apic.disable_irq(i);
                    }
                    // info!("ioapic table entry [{}]\n{:?}", i, io_apic.table_entry(i));
                }
                IO_APIC.call_once(|| SpinlockIrqSave::new(io_apic));
            }
        }
        crate::util::barrier();

        info!("InterruptController apic init ok");
    }

    fn enable(int: Interrupt) {
        info!("InterruptController apic enable int {}", int);
        if int < TIMER_INTERRUPT_NUMBER as _ {
            unsafe {
                IO_APIC.get_mut().unwrap().lock().enable_irq(int as u8);
            }
        }
    }
    fn disable(int: Interrupt) {
        info!("InterruptController apic disable int {}", int);
        if int < TIMER_INTERRUPT_NUMBER as _ {
            unsafe {
                IO_APIC.get_mut().unwrap().lock().disable_irq(int as u8);
            }
        }
    }

    fn fetch() -> Option<Interrupt> {
        unimplemented!();
    }
    fn finish(_int: Interrupt) {
        unsafe { local_apic().end_of_interrupt() }
    }
}

pub type Interrupt = usize;

pub fn local_apic<'a>() -> &'a mut LocalApic {
    // It's safe as LAPIC is per-cpu.
    unsafe { LOCAL_APIC.as_mut().unwrap() }
}

/// For smp support, current not used.
/// Todo: support smp.
#[allow(unused)]
pub(super) fn raw_apic_id(id_u8: u8) -> u32 {
    if unsafe { IS_X2APIC } {
        id_u8 as u32
    } else {
        (id_u8 as u32) << 24
    }
}

fn cpu_has_x2apic() -> bool {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.has_x2apic(),
        None => false,
    }
}
