use spin::Mutex;
use spin::Lazy;

use bitflags::bitflags;
use bitmaps::Bitmap;

pub type ZoneId = usize;

pub const ZONE_ID_PRIVILEGED: ZoneId = 0x0;
pub const ZONE_ID_SHARED: ZoneId = 0xf;

// By default, only shared zone(with pkey 15) can be accessed.
pub const PKRU_DEFAULT: u32 = 0x3fff;
pub const PKRU_PRIVILEGED: u32 = 0x0;

// https://man7.org/linux/man-pages/man7/pkeys.7.html

// The PKRU register (protection-key rights for user pages) is a 32-bit register with the following format:
// for each i (0 ≤ i ≤ 15)
// PKRU[2i] is the access-disable bit for protection key i (ADi)
// PKRU[2i+1] is the write-disable bitfor protection key i (WDi).
pub enum PkeyPerm {
    NoAccess = 0b11,
    ReadOnly = 0b10,
    ReadWrite = 0b00,
}

pub fn switch_to_privilege_pkru() -> u32 {
    let ori_pkru = rdpkru();
    wrpkru(PKRU_PRIVILEGED);
    ori_pkru
}

pub fn switch_from_privilege_pkru(ori_pkru: u32) {
    wrpkru(ori_pkru);
}

/// Get current PKRU register value.
pub fn rdpkru() -> u32 {
    crate::arch::mpk::rdpkru()
}

/// Set current PKRU register value.
pub fn wrpkru(val: u32) {
    crate::arch::mpk::wrpkru(val)
}

pub fn dump_pkru() {
    println!("current pkru {:#x}", rdpkru());
}



// The PKRU register (protection-key rights for user pages) is a 32-bit register with the following format:
// for each i (0 ≤ i ≤ 15)
// PKRU[2i] is the access-disable bit for protection key i (ADi)
// PKRU[2i+1] is the write-disable bitfor protection key i (WDi).
bitflags! {
    pub struct ZoneKeys: u32 {
        /// Use a relative timeout
        const KEY_0_NOREAD = 0b01 << (2*0);
        const KEY_0_NOWRITE = 0b10 << (2*0);

        const KEY_1_NOREAD = 0b01 << (2*1);
        const KEY_1_NOWRITE = 0b10 << (2*1);

        const KEY_2_NOREAD = 0b01 << (2*2);
        const KEY_2_NOWRITE = 0b10 << (2*2);

        const KEY_3_NOREAD = 0b01 << (2*3);
        const KEY_3_NOWRITE = 0b10 << (2*3);

        const KEY_4_NOREAD = 0b01 << (2*4);
        const KEY_4_NOWRITE = 0b10 << (2*4);

        const KEY_5_NOREAD = 0b01 << (2*5);
        const KEY_5_NOWRITE = 0b10 << (2*5);

        const KEY_6_NOREAD = 0b01 << (2*6);
        const KEY_6_NOWRITE = 0b10 << (2*6);

        const KEY_7_NOREAD = 0b01 << (2*7);
        const KEY_7_NOWRITE = 0b10 << (2*7);

        const KEY_8_NOREAD = 0b01 << (2*8);
        const KEY_8_NOWRITE = 0b10 << (2*8);

        const KEY_9_NOREAD = 0b01 << (2*9);
        const KEY_9_NOWRITE = 0b10 << (2*9);

        const KEY_10_NOREAD = 0b01 << (2*10);
        const KEY_10_NOWRITE = 0b10 << (2*10);

        const KEY_11_NOREAD = 0b01 << (2*11);
        const KEY_11_NOWRITE = 0b10 << (2*11);

        const KEY_12_NOREAD = 0b01 << (2*12);
        const KEY_12_NOWRITE = 0b10 << (2*12);

        const KEY_13_NOREAD = 0b01 << (2*13);
        const KEY_13_NOWRITE = 0b10 << (2*13);

        const KEY_14_NOREAD = 0b01 << (2*14);
        const KEY_14_NOWRITE = 0b10 << (2*14);

        const KEY_15_NOREAD = 0b01 << (2*15);
        const KEY_15_NOWRITE = 0b10 << (2*15);

        const KEY_0_NOACCESS = Self::KEY_0_NOREAD.bits | Self::KEY_0_NOWRITE.bits;
        const KEY_1_NOACCESS = Self::KEY_1_NOREAD.bits | Self::KEY_1_NOWRITE.bits;
        const KEY_2_NOACCESS = Self::KEY_2_NOREAD.bits | Self::KEY_2_NOWRITE.bits;
        const KEY_15_NOACCESS = Self::KEY_15_NOREAD.bits | Self::KEY_15_NOWRITE.bits;
    }
}

static GLOBAL_ZONES: Lazy<Mutex<Bitmap<16>>> = Lazy::new(|| Mutex::new(Bitmap::<16>::new()));

impl ZoneKeys {
    pub fn as_pkru(&self) -> u32 {
        self.bits
    }
}

impl From<ZoneId> for ZoneKeys {
    fn from(zone_id: ZoneId) -> Self {
        if zone_id >= ZONE_ID_SHARED {
            return Self { bits: PKRU_DEFAULT };
        }
        let mut pkru = PKRU_DEFAULT;
        pkru &= !(1 << (zone_id * 2));
        pkru &= !(1 << ((zone_id * 2) + 1));

        Self { bits: pkru }
    }
}
// Use bitmap to manage zone allocation & deallocation.
pub fn zone_init() {
    let mut global_zone = GLOBAL_ZONES.lock();
    global_zone.set(ZONE_ID_SHARED, true);
    global_zone.set(ZONE_ID_PRIVILEGED, true);
}

pub fn zone_alloc() -> Option<ZoneId> {
    let mut global_zone = GLOBAL_ZONES.lock();
    let allocated_zone = global_zone.first_false_index().unwrap_or_else(|| {
        warn!("No free zones, use ZONE_ID_SHARED {ZONE_ID_SHARED} by default");
        ZONE_ID_SHARED
    });

    global_zone.set(allocated_zone, true);

    return Some(allocated_zone);
}

pub fn zone_free(zone_id: ZoneId) {
    assert!(zone_id < ZONE_ID_PRIVILEGED, "unexpected zone id");
    let mut global_zone = GLOBAL_ZONES.lock();
    global_zone.set(zone_id, false);
}

