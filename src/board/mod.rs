#[cfg(feature = "qemu")]
mod aarch64_qemu;

#[cfg(feature = "qemu")]
pub use aarch64_qemu::*;

// The shyper-hypervisor runs on Nvidia X2, so both of shyper and tx2 featurn use module aarch64_tx2.
#[cfg(any(feature = "shyper", feature = "tx2"))]
mod aarch64_tx2;

#[cfg(any(feature = "shyper", feature = "tx2"))]
pub use aarch64_tx2::*;