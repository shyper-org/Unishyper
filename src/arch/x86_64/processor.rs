use raw_cpuid::CpuId;
use x86_64::registers::control::{Cr4, Cr4Flags};

pub fn configure() {
    let cpuid = CpuId::new();
    let feature_info = cpuid
        .get_feature_info()
        .expect("CPUID Feature Info not available!");
    let extended_feature_info = cpuid
        .get_extended_feature_info()
        .expect("CPUID Extended Feature Info not available!");

    let core_id = feature_info.initial_local_apic_id() as usize;
    // println!("Processor [{}]", core_id);

    let cr4_flags = Cr4::read();
    debug!("Processor CR4 flags [{:?}]", cr4_flags);

    if extended_feature_info.has_pku() {
        info!("Processor [{}] supports PKU", core_id);
        unsafe {
            Cr4::update(|f| f.insert(Cr4Flags::PROTECTION_KEY_USER));
            // Cr4::update(|f| f.insert(Cr4Flags::PROTECTION_KEY_SUPERVISOR));
        }
    }
    if extended_feature_info.has_fsgsbase() {
        info!("Processor [{}] supports FSGSBASE", core_id);
        unsafe {
            Cr4::update(|f| f.insert(Cr4Flags::FSGSBASE));
        }
    }

    let raw_cr4 = Cr4::read_raw();
    let cr4_flags = Cr4::read();
    debug!("Processor cr4 {:#x} flags [{:?}]", raw_cr4, cr4_flags);
}
