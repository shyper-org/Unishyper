pub const WORD_SHIFT: usize = 3;
pub const WORD_SIZE: usize = 1 << WORD_SHIFT;

extern "C" {
    pub fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8;
    pub fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    pub fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32;
    pub fn strlen(s: *const core::ffi::c_char) -> usize;
}

/// Memmove
///
/// Copy N bytes of memory from src to dest. The memory areas may overlap.
///
/// This faster implementation works by copying bytes not one-by-one, but in
/// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if src < dest as *const u8 {
        let n_usize: usize = n / WORD_SIZE; // Number of word sized groups
        let mut i: usize = n_usize * WORD_SIZE;

        // Copy `WORD_SIZE` bytes at a time
        while i != 0 {
            i -= WORD_SIZE;
            *((dest as usize + i) as *mut usize) = *((src as usize + i) as *const usize);
        }

        let mut i: usize = n;

        // Copy 1 byte at a time
        while i != n_usize * WORD_SIZE {
            i -= 1;
            *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
        }
    } else {
        let n_usize: usize = n / WORD_SIZE; // Number of word sized groups
        let mut i: usize = 0;

        // Copy `WORD_SIZE` bytes at a time
        let n_fast = n_usize * WORD_SIZE;
        while i < n_fast {
            *((dest as usize + i) as *mut usize) = *((src as usize + i) as *const usize);
            i += WORD_SIZE;
        }

        // Copy 1 byte at a time
        while i < n {
            *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
            i += 1;
        }
    }

    dest
}