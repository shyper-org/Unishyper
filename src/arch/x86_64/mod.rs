use crate::libs::traits::Address;
use crate::libs::traits::ArchTrait;

mod context_frame;
mod exception;
mod gdt;
pub mod irq;
pub mod page_table;
mod processor;

pub const PHYSICAL_MEMORY_OFFSET: u64 = 0xFFFF_8000_0000_0000;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const MACHINE_SIZE: usize = core::mem::size_of::<usize>();

pub const MAX_VIRTUAL_ADDRESS: usize = usize::MAX;
pub const MIN_USER_VIRTUAL_ADDRESS: usize = 0x0000_0010_0000_0000;
pub const MAX_USER_VIRTUAL_ADDRESS: usize = 0x0000_007F_FFFF_FFFF;

pub const MAX_PAGE_NUMBER: usize = MAX_VIRTUAL_ADDRESS / PAGE_SIZE;



/// The virtual address offset from which physical memory is mapped, as described in
/// https://os.phil-opp.com/paging-implementation/#map-the-complete-physical-memory
/// It's determined by rboot in rboot.conf.
const PA2KVA: usize = 0xFFFF_8000_0000_0000;
const KVA2PA: usize = 0x0000_7FFF_FFFF_FFFF;

impl Address for usize {
    fn pa2kva(&self) -> usize {
        *self | PA2KVA
    }
    fn kva2pa(&self) -> usize {
        *self & KVA2PA
    }
}

pub type ContextFrame = context_frame::X86_64TrapContextFrame;
pub type ThreadContext = context_frame::ThreadContext;

pub use exception::irq_install_handler;
pub use exception::init_idt;

pub use gdt::Cpu;

use rboot::BootInfo;
static mut BOOT_INFO: Option<&'static BootInfo> = None;

pub fn boot_info() -> &'static BootInfo {
    unsafe { BOOT_INFO.as_ref().unwrap() }
}

pub fn cpu_id() -> usize {
    raw_cpuid::CpuId::new()
        .get_feature_info()
        .unwrap()
        .initial_local_apic_id() as usize
}

#[no_mangle]
#[link_section = ".text.start"]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    // println!("\nentering x86_64 entry...\n");
    // dump_boot_info_memory_layout(boot_info);

    // Test boot time
    // unsafe {
    //     let start_cycle = core::arch::x86_64::_rdtsc();
    //     // println!("\n start cycle {start_cycle}");
    //     crate::START_CYCLE = start_cycle;
    // }

    unsafe {
        BOOT_INFO = Some(boot_info);
    }

    // Jump to loader main.
    let core_id = cpu_id();
    // println!("\nentering loader_main on cpu {}\n", core_id);
    crate::loader_main(core_id);
    loop {}
}

#[allow(unused)]
pub fn dump_boot_info_memory_layout(boot_info: &'static BootInfo) {
    println!("Dump boot_info memory layout:");
    for (idx, m) in boot_info.memory_map.into_iter().enumerate() {
        println!(
            "[{:>2}] [{:#x}-{:#x}] {:?} {:#x} {}",
            idx,
            m.phys_start,
            m.phys_start + m.page_count * 0x1000,
            m.ty,
            m.page_count * 0x1000,
            m.page_count
        );
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Exit qemu
/// See: https://wiki.osdev.org/Shutdown
/// Must run qemu with `-device isa-debug-exit`
/// The error code is `value written to 0x501` *2 +1, so it should be odd when non-zero
#[allow(unused)]
pub unsafe fn exit_in_qemu(error_code: u8) -> ! {
    use x86_64::instructions::port::Port;
    if error_code == 0 {
        Port::new(0xB004).write(0x2000 as u16);
    } else {
        assert_eq!(error_code & 1, 1, "error code should be odd");
        Port::new(0x501).write((error_code - 1) / 2);
    }
    unreachable!()
}

#[allow(unused)]
pub unsafe fn reboot() -> ! {
    use x86_64::instructions::port::Port;
    Port::new(0x64).write(0xfeu8);
    unreachable!()
}

pub struct Arch;

impl ArchTrait for Arch {
    fn exception_init() {
        x86_64::instructions::interrupts::disable();
        processor::configure();
        gdt::add_current_core();
        exception::load_idt();
        // x86_64::instructions::interrupts::enable();
        info!("exception init success!");
    }

    fn page_table_init() {
        debug!("init page table for x86_64");
        page_table::init();
    }

    fn invalidate_tlb() {}

    fn wait_for_interrupt() {
        x86_64::instructions::hlt()
    }

    fn nop() {
        x86_64::instructions::nop()
    }

    fn fault_address() -> usize {
        0
    }

    #[inline(always)]
    fn core_id() -> usize {
        cpu_id()
    }

    fn curent_privilege() -> usize {
        0
    }
    #[inline(always)]
    fn pop_context_first(ctx: usize) -> ! {
        // #[cfg(feature = "zone")]
        // mpk::wrpkru(mpk::pkru_of_zone_id(1));
        unsafe { context_frame::_pop_context_first(ctx) }
        loop {}
    }

    fn set_thread_id(_tid: u64) {}

    fn get_tls_ptr() -> *const u8 {
        0xDEAD_BEEF as *const u8 as *const u8
    }

    fn set_tls_ptr(_tls_ptr: u64) {}
}
