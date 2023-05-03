#[cfg(all(target_arch = "aarch64", feature = "qemu"))]
mod aarch64_qemu;

#[cfg(all(target_arch = "aarch64", feature = "qemu"))]
pub use aarch64_qemu::*;

// The shyper-hypervisor runs on Nvidia X2, so both of shyper and tx2 featurn use module aarch64_tx2.
#[cfg(all(target_arch = "aarch64", any(feature = "shyper", feature = "tx2")))]
mod aarch64_tx2;

#[cfg(all(target_arch = "aarch64", any(feature = "shyper", feature = "tx2")))]
pub use aarch64_tx2::*;

#[cfg(all(target_arch = "x86_64", feature = "qemu"))]
mod x86_64qemu;

#[cfg(all(target_arch = "x86_64", feature = "qemu"))]
pub use x86_64qemu::*;