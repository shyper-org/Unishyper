use crate::mm::interface::*;
use core::fmt::{Display, Formatter};

use crate::arch::PAGE_SHIFT;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EntryAttribute {
    writable: bool,
    user: bool,
    device: bool,
    k_executable: bool,
    u_executable: bool,
    copy_on_write: bool,
    shared: bool,
    block: bool,
    #[cfg(feature = "mpk")]
    zone_id: usize,
}

impl PageTableEntryAttrTrait for EntryAttribute {
    fn writable(&self) -> bool {
        self.writable
    }

    fn k_executable(&self) -> bool {
        self.k_executable
    }

    fn u_executable(&self) -> bool {
        self.u_executable
    }

    fn u_readable(&self) -> bool {
        self.user
    }

    fn u_copy_on_write(&self) -> bool {
        self.copy_on_write
    }

    fn u_shared(&self) -> bool {
        self.shared
    }

    fn device(&self) -> bool {
        self.device
    }

    fn block(&self) -> bool {
        self.block
    }

    fn set_block(&self) -> Self {
        EntryAttribute {
            writable: self.writable,
            user: self.user,
            device: self.device,
            k_executable: self.k_executable,
            u_executable: self.u_executable,
            copy_on_write: self.copy_on_write,
            shared: self.shared,
            #[cfg(feature = "mpk")]
            zone_id: self.zone_id,
            block: true,
        }
    }

    fn copy_on_write(&self) -> bool {
        self.copy_on_write
    }

    fn new(
        writable: bool,
        user: bool,
        device: bool,
        k_executable: bool,
        u_executable: bool,
        copy_on_write: bool,
        shared: bool,
        block: bool,
    ) -> Self {
        EntryAttribute {
            writable,
            user,
            device,
            k_executable,
            u_executable,
            copy_on_write,
            shared,
            #[cfg(feature = "mpk")]
            zone_id: 0,
            block,
        }
    }

    fn kernel_device() -> Self {
        EntryAttribute {
            writable: true,
            user: false,
            device: true,
            k_executable: false,
            u_executable: false,
            copy_on_write: false,
            shared: false,
            #[cfg(feature = "mpk")]
            zone_id: 0,
            block: false,
        }
    }

    fn user_default() -> Self {
        EntryAttribute {
            writable: true,
            user: true,
            device: false,
            k_executable: false,
            u_executable: true,
            copy_on_write: false,
            shared: false,
            #[cfg(feature = "mpk")]
            zone_id: 0,
            block: false,
        }
    }

    fn user_2mb() -> Self {
        EntryAttribute {
            writable: true,
            user: true,
            device: false,
            k_executable: false,
            u_executable: true,
            copy_on_write: false,
            shared: false,
            #[cfg(feature = "mpk")]
            zone_id: 0,
            block: true,
        }
    }

    fn user_readonly() -> Self {
        EntryAttribute {
            writable: false,
            user: true,
            device: false,
            k_executable: false,
            u_executable: false,
            copy_on_write: false,
            shared: false,
            #[cfg(feature = "mpk")]
            zone_id: 0,
            block: false,
        }
    }

    fn user_executable() -> Self {
        EntryAttribute {
            writable: false,
            user: true,
            device: false,
            k_executable: false,
            u_executable: true,
            copy_on_write: false,
            shared: false,
            #[cfg(feature = "mpk")]
            zone_id: 0,
            block: false,
        }
    }

    fn user_data() -> Self {
        EntryAttribute {
            writable: true,
            user: true,
            device: false,
            k_executable: false,
            u_executable: false,
            copy_on_write: false,
            shared: false,
            #[cfg(feature = "mpk")]
            zone_id: 0,
            block: false,
        }
    }

    fn user_device() -> Self {
        EntryAttribute {
            writable: true,
            user: true,
            device: true,
            k_executable: false,
            u_executable: false,
            copy_on_write: false,
            shared: false,
            #[cfg(feature = "mpk")]
            zone_id: 0,
            block: false,
        }
    }

    fn filter(&self) -> Self {
        EntryAttribute {
            writable: self.writable,
            user: true,
            device: false,
            k_executable: false,
            u_executable: self.u_executable,
            copy_on_write: self.copy_on_write,
            shared: self.shared,
            #[cfg(feature = "mpk")]
            zone_id: 0,
            block: false,
        }
    }
}

#[cfg(feature = "mpk")]
impl PageTableEntryAttrZoneTrait for EntryAttribute {
    fn set_zone(&mut self, zone_id: usize) {
        self.zone_id = zone_id;
    }
    fn get_zone_id(&self) -> usize {
        self.zone_id as usize
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Entry {
    attribute: EntryAttribute,
    pa: usize,
}

#[allow(unused)]
impl Entry {
    pub fn new(attribute: EntryAttribute, pa: usize) -> Self {
        Entry { attribute, pa }
    }
    pub fn attribute(&self) -> EntryAttribute {
        self.attribute
    }
    pub fn pa(&self) -> usize {
        self.pa
    }
    pub fn ppn(&self) -> usize {
        self.pa >> PAGE_SHIFT
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "PageTableEntry [{:016x}] {:?}", self.pa, self.attribute)
    }
}
