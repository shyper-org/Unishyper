use core::ops::Range;

use gimli::BaseAddresses;
use spin::Lazy;
use spin::Once;
use xmas_elf::*;
use xmas_elf::sections::SectionData;

// Store kernel's elf image.
extern "C" {
    static ELF_IMAGE: [u8; 0x40000000];
}

static BASE_ADDRESSES: Lazy<BaseAddresses> = Lazy::new(|| {
    let eh_frame = section_by_name(".eh_frame").unwrap();
    let eh_frame_hdr = section_by_name(".eh_frame_hdr").unwrap();
    let text = section_by_name(".text").unwrap();
    debug!(
        "eh_frame: range [0x{:016x} - 0x{:016x}]",
        eh_frame.start, eh_frame.end
    );
    debug!(
        "eh_frame_hdr: range [0x{:016x} - 0x{:016x}]",
        eh_frame_hdr.start, eh_frame_hdr.end
    );
    debug!("text: range [0x{:016x} - 0x{:016x}]", text.start, text.end);
    BaseAddresses::default()
        .set_eh_frame_hdr(eh_frame_hdr.start)
        .set_eh_frame(eh_frame.start)
        .set_text(text.start)
});

static ELF_FILE: Lazy<ElfFile> =
    Lazy::new(|| ElfFile::new(unsafe { &ELF_IMAGE }).expect("failed to parse elf file"));

pub fn base_addresses() -> BaseAddresses {
    BASE_ADDRESSES.clone()
}

static EH_FRAME: Once<Range<u64>> = Once::new();

fn eh_frame() -> Range<u64> {
    match EH_FRAME.get() {
        Some(r) => r,
        None => EH_FRAME.call_once(|| section_by_name(".eh_frame").unwrap()),
    }
    .clone()
}

pub fn eh_frame_slice() -> &'static [u8] {
    let eh_frame = eh_frame();
    unsafe {
        core::slice::from_raw_parts(
            eh_frame.start as usize as *const u8,
            (eh_frame.end - eh_frame.start) as usize,
        )
    }
}

#[no_mangle]
fn section_by_name(name: &'static str) -> Option<Range<u64>> {
    let elf = &ELF_FILE;
    for section_header in elf.section_iter() {
        // println!("section_header {:?}", section_header);
        if let Ok(section_name) = section_header.get_name(elf) {
            if section_name == name {
                return Some(
                    section_header.address()..(section_header.address() + section_header.size()),
                );
            }
        }
    }
    warn!("Get section by name {} from ELF file failed!!!", name);
    None
}

pub fn section_by_addr(addr: usize) -> Option<&'static [u8]> {
    let elf = &ELF_FILE;
    for section_header in elf.section_iter() {
        if addr >= section_header.address() as usize
            && addr < (section_header.address() + section_header.size()) as usize
        {
            match section_header.get_data(elf) {
                Ok(x) => {
                    return match x {
                        SectionData::Undefined(r) => Some(r),
                        _ => None,
                    };
                }
                Err(_) => {}
            }
        }
    }
    warn!("Get section by addr {:#x} from ELF file failed!!!", addr);
    None
}
