mod gdt;
mod interface;
mod exception;
pub mod irq;
pub mod page_table;
pub use interface::*;
mod context_frame;
pub mod mpk;
mod processor;

pub use context_frame::{
    yield_to, set_thread_id, set_tls_ptr, get_tls_ptr, pop_context_first,
};

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
    println!("\nentering x86_64 entry...\n");
    // dump_boot_info_memory_layout(boot_info);

    unsafe {
        BOOT_INFO = Some(boot_info);
    }

    // Jump to loader main.
    let core_id = cpu_id();
    println!("\nentering loader_main on cpu {}\n", core_id);
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
