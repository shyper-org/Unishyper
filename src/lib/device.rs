use core::ops::Range;

pub enum Device {
    Virtio(VirtioDevice),
    Unknown,
}

#[derive(Debug)]
pub struct VirtioDevice {
    pub name: &'static str,
    pub registers: Range<usize>,
    pub interrupts: usize,
}

impl VirtioDevice {
    #[allow(dead_code)]
    pub fn new(name: &'static str, registers: Range<usize>, interrupts: usize) -> Self {
        Self {
            name,
            registers,
            interrupts,
        }
    }
}
