cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        // Current X86_64 only support QEMU platform `qemu-system-x86_64`.
        mod x86_64qemu;
        pub use x86_64qemu::*;
    } else if #[cfg(target_arch = "riscv64")] {
        // Current RISCV only support QEMU platform `qemu-system-riscv64`.
        mod riscv64_qemu;
        pub use riscv64_qemu::*;
        // Pending: maybe port to K210 in the future.
    } else {
        // By default, target architecture is aarch64.
        cfg_if::cfg_if! {
            if #[cfg(feature = "tx2")] {
                // Nvidia Tegra X2 platform.
                mod aarch64_tx2;
                pub use aarch64_tx2::*;
            } else {
                // QEMU platform `qemu-system-aarch64`.
                mod aarch64_qemu;
                pub use aarch64_qemu::*;
            }
        }
    }
}
