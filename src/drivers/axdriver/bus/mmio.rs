#[allow(unused_imports)]
use super::super::{prelude::*, AllDevices};

use crate::board::devices;

impl AllDevices {
    pub(crate) fn probe_bus_devices(&mut self) {
        // TODO: parse device tree
        for d in devices() {
            for_each_drivers!(type Driver, {
                info!(
                    "Try to probe device at [PA:{:#x}, PA:{:#x}), irq {:#x}",
                    d.range().start, d.range().end,
                    d.irq_id(),
                );
                if let Some(dev) = Driver::probe_mmio(d.range().start, d.range().end - d.range().start) {
                    info!(
                        "registered a new {:?} device at [PA:{:#x}, PA:{:#x}): {:?} irq {:#x}",
                        dev.device_type(),
                        d.range().start, d.range().end,
                        dev.device_name(),
                        d.irq_id(),
                    );
                    self.add_device(dev, Some(d.irq_id() as u32));
                    continue; // skip to the next device
                } else {
                    warn!(
                        "Device at [PA:{:#x}, PA:{:#x}), irq {:#x} not exist!",
                        d.range().start, d.range().end,
                        d.irq_id(),
                    );
                    continue; // skip to the next device
                }
            });
        }
    }
}
