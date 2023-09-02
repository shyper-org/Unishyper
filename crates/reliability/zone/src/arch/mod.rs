cfg_if::cfg_if!(
    if #[cfg(target_arch = "x86_64")] {
        mod mpk;
        pub use mpk::*;
    } else {
        /// Get current PKRU register value.
        pub fn rdpkru() -> u32 {
            0
        }

        /// Set current PKRU register value.
        pub fn wrpkru(_val: u32) {}
    }
);
