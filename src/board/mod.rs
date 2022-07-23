#[cfg(feature = "qemu")]
mod aarch64_qemu;

#[cfg(feature = "qemu")]
pub use aarch64_qemu::*;

#[cfg(feature = "shyper")]
mod aarch64_shyper;

#[cfg(feature = "shyper")]
pub use aarch64_shyper::*;