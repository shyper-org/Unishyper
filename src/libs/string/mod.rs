pub const WORD_SHIFT: usize = 3;
pub const WORD_SIZE: usize = 1 << WORD_SHIFT;

extern "C" {
    // pub fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8;
    // pub fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    pub fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32;
    pub fn strlen(s: *const core::ffi::c_char) -> usize;
}

/// To support core::intrinsics::sqrtf64;
/// See https://gitee.com/unishyper/rust-fstest
/// Todo: To be removed.
#[cfg(all(not(feature = "std"), feature = "fs"))]
#[no_mangle]
pub extern "C" fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

cfg_if::cfg_if! {
  if #[cfg(all(target_arch = "aarch64", feature = "asm"))] {
    extern "C" {
      pub fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8;
      pub fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    }
    core::arch::global_asm!(include_str!("aarch64/memset.S"));
    core::arch::global_asm!(include_str!("aarch64/memcpy.S"));

  } else if #[cfg(all(target_arch = "riscv64", feature = "asm"))] {
    extern "C" {
      pub fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8;
      pub fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    }
    core::arch::global_asm!(include_str!("riscv64/memset.S"));
    core::arch::global_asm!(include_str!("riscv64/memcpy.S"));

  } else {
    extern "C" {
      pub fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8;
      pub fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    }
    // /// Memset
    // ///
    // /// Fill a block of memory with a specified value.
    // ///
    // /// This faster implementation works by setting bytes not one-by-one, but in
    // /// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
    // #[no_mangle]
    // pub unsafe extern fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    //   let c: usize = core::mem::transmute([c as u8; WORD_SIZE]);
    //   let n_usize: usize = n/WORD_SIZE;

    //   let mut i: usize = 0;

    //   // Set `WORD_SIZE` bytes at a time
    //   let n_fast = n_usize*WORD_SIZE;
    //   while i < n_fast {
    //     *((dest as usize + i) as *mut usize) = c;
    //     i += WORD_SIZE;
    //   }

    //   let c = c as u8;

    //   // Set 1 byte at a time
    //   while i < n {
    //     *((dest as usize + i) as *mut u8) = c;
    //     i += 1;
    //   }

    //   dest
    // }

    // /// Memcpy
    // ///
    // /// Copy N bytes of memory from one location to another.
    // ///
    // /// This faster implementation works by copying bytes not one-by-one, but in
    // /// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
    // #[no_mangle]
    // pub unsafe extern fn memcpy(dest: *mut u8, src: *const u8,
    //                             n: usize) -> *mut u8 {

    //   let n_usize: usize = n/WORD_SIZE; // Number of word sized groups
    //   let mut i: usize = 0;

    //   // Copy `WORD_SIZE` bytes at a time
    //   let n_fast = n_usize*WORD_SIZE;
    //   while i < n_fast {
    //     *((dest as usize + i) as *mut usize) =
    //       *((src as usize + i) as *const usize);
    //     i += WORD_SIZE;
    //   }

    //   // Copy 1 byte at a time
    //   while i < n {
    //     *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
    //     i += 1;
    //   }

    //   dest
    // }
  }
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

// /// Memcmp
// ///
// /// Compare two blocks of memory.
// ///
// /// This faster implementation works by comparing bytes not one-by-one, but in
// /// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
// #[no_mangle]
// pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
//     let n_usize: usize = n / WORD_SIZE;
//     let mut i: usize = 0;

//     let n_fast = n_usize * WORD_SIZE;
//     while i < n_fast {
//         let a = *((s1 as usize + i) as *const usize);
//         let b = *((s2 as usize + i) as *const usize);
//         if a != b {
//             let n: usize = i + WORD_SIZE;
//             // Find the one byte that is not equal
//             while i < n {
//                 let a = *((s1 as usize + i) as *const u8);
//                 let b = *((s2 as usize + i) as *const u8);
//                 if a != b {
//                     return a as i32 - b as i32;
//                 }
//                 i += 1;
//             }
//         }
//         i += WORD_SIZE;
//     }

//     while i < n {
//         let a = *((s1 as usize + i) as *const u8);
//         let b = *((s2 as usize + i) as *const u8);
//         if a != b {
//             return a as i32 - b as i32;
//         }
//         i += 1;
//     }

//     0
// }
