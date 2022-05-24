use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU16, Ordering};

use spin::Mutex;

pub type Asid = u16;
pub type Error = usize;

pub const ERROR_INVARG: usize = 1;
pub const ERROR_OOM: usize = 2;
pub const ERROR_MEM_NOT_MAP: usize = 3;
pub const ERROR_INTERNAL: usize = 4;
pub const ERROR_DENIED: usize = 5;
pub const ERROR_HOLD_ON: usize = 6;
pub const ERROR_OOR: usize = 7;
pub const ERROR_PANIC: usize = 8;

#[derive(Debug)]
struct Inner {
    asid: Asid,
    //   page_table: PageTable,
    exception_handler: Mutex<Option<usize>>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        trace!("Drop AS{}", self.asid);
    }
}

#[derive(Debug, Clone)]
pub struct AddressSpace(Arc<Inner>);

impl PartialEq for AddressSpace {
    fn eq(&self, other: &Self) -> bool {
        self.0.asid == other.0.asid
    }
}

impl AddressSpace {
    pub fn asid(&self) -> Asid {
        self.0.asid
    }

    pub fn exception_handler(&self) -> Option<usize> {
        let lock = self.0.exception_handler.lock();
        lock.clone()
    }

    pub fn set_exception_handler(&self, handler: Option<usize>) {
        let mut lock = self.0.exception_handler.lock();
        *lock = handler;
    }
}

static ASID_ALLOCATOR: AtomicU16 = AtomicU16::new(1);

fn new_asid() -> Asid {
    ASID_ALLOCATOR.fetch_add(1, Ordering::Relaxed)
}

static ADDRESS_SPACE_MAP: Mutex<BTreeMap<Asid, AddressSpace>> = Mutex::new(BTreeMap::new());

pub fn address_space_alloc() -> Result<AddressSpace, Error> {
    let id = new_asid();
    if id == 0 {
        return Err(ERROR_OOR);
    }
    let a = AddressSpace(
        Arc::try_new(Inner {
            asid: id,
            exception_handler: Mutex::new(None),
        })
        .map_err(|_| ERROR_OOM)?,
    );
    let mut map = ADDRESS_SPACE_MAP.lock();
    map.insert(id, a.clone());
    Ok(a)
}

pub fn address_space_lookup(asid: Asid) -> Option<AddressSpace> {
    let map = ADDRESS_SPACE_MAP.lock();
    match map.get(&asid) {
        Some(a) => Some(a).cloned(),
        None => None,
    }
}

pub fn address_space_destroy(a: AddressSpace) {
    trace!("Destroy AS{}", a.asid());
    let mut map = ADDRESS_SPACE_MAP.lock();
    map.remove(&a.asid());
}

pub fn load_image(elf: &'static [u8]) -> (AddressSpace, usize) {
    // let icntr = crate::lib::timer::current_cycle();
    let a = address_space_alloc().unwrap();
    // let icntr2 = crate::lib::timer::current_cycle();
    // info!("as create cycle {}", icntr2 - icntr);
    let entry = 0;
    (a, entry)
}
