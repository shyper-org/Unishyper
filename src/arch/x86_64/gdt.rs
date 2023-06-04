use x86_64::instructions::tables::load_tss;
use x86_64::instructions::segmentation::{CS, SS, Segment};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub struct Cpu {
    gdt: GlobalDescriptorTable,
    tss: TaskStateSegment,
    double_fault_stack: [u8; 0x100],
    id: usize,
}

impl Cpu {
    pub const fn new() -> Self {
        Cpu {
            gdt: GlobalDescriptorTable::new(),
            tss: TaskStateSegment::new(),
            double_fault_stack: [0u8; 0x100],
            id: 0,
        }
    }

    fn init(&'static mut self) {
        self.id = super::cpu_id();

        debug!("Core[{}] init gdt and tss", self.id);

        let stack_top = VirtAddr::new(self.double_fault_stack.as_ptr() as u64 + 0x100);
        self.tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_top;

        debug!(
            "DOUBLE_FAULT_STACK stack_start {:#x} stack_end {:#x}",
            self.double_fault_stack.as_ptr() as u64,
            stack_top
        );

        let code_selector = self.gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = self.gdt.add_entry(Descriptor::kernel_data_segment());
        let tss_selector = self.gdt.add_entry(Descriptor::tss_segment(&self.tss));

        debug!("code_selector {:?}", code_selector);
        debug!("data_selector {:?}", data_selector);
        debug!("tss_selector {:?}", tss_selector);

        self.gdt.load();
        unsafe {
            CS::set_reg(code_selector);
            SS::set_reg(data_selector);
            load_tss(tss_selector);
        }
    }
}

pub fn add_current_core() {
    let cpu = crate::libs::cpu::cpu().get_cpu_data();
    cpu.init();
}
