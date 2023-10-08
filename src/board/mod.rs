// QEMU platform `qemu-system-aarch64`.
#[cfg_attr(
    all(target_arch = "aarch64", feature = "qemu"),
    path = "aarch64_qemu.rs"
)]
// Nvidia Tegra X2 platform.
#[cfg_attr(all(target_arch = "aarch64", feature = "tx2"), path = "aarch64_tx2.rs")]
// Shyper Hypervisor.
#[cfg_attr(
    all(target_arch = "aarch64", feature = "shyper"),
    path = "aarch64_tx2.rs"
)]
#[cfg_attr(
    all(target_arch = "aarch64", feature = "rk3588"),
    path = "aarch64_tx2.rs"
)]
// QEMU platform `qemu-system-x86_64`.
#[cfg_attr(all(target_arch = "x86_64", feature = "qemu"), path = "x86_64_qemu.rs")]
// QEMU platform `qemu-system-riscv64`.
#[cfg_attr(
    all(target_arch = "riscv64", feature = "qemu"),
    path = "riscv64_qemu.rs"
)]
// Kendryte K210.
#[cfg_attr(
    all(target_arch = "riscv64", feature = "k210"),
    path = "riscv64_k210.rs"
)]
mod specific_board;

pub use specific_board::*;
