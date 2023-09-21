use core::ops::Range;

#[derive(Debug)]
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

impl Device {
    pub fn name(&self) -> &'static str {
        match self {
            Device::Virtio(device) => device.name,
            Device::Unknown => "unknown device",
        }
    }

    pub fn range(&self) -> Range<usize> {
        match self {
            Device::Virtio(device) => device.registers.start..device.registers.end,
            Device::Unknown => 0..0,
        }
    }

    pub fn irq_id(&self) -> usize {
        match self {
            Device::Virtio(device) => device.interrupts,
            Device::Unknown => 0,
        }
    }
}
