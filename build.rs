use std::{env, error::Error, collections::HashMap};

fn arch_machine_supported(arch: &str, machine: &str) -> bool {
    let map = HashMap::from([
        ("x86_64", vec!["qemu"]),
        ("riscv64", vec!["k210", "qemu"]),
        ("aarch64", vec!["pi4", "tx2", "rk3588", "qemu", "shyper"]),
    ]);
    if let Some(machine_supported) = map.get(arch) {
        if machine_supported.contains(&machine) {
            return true;
        }
    }
    false
}

fn main() -> Result<(), Box<dyn Error>> {
    let arch = match env::var("ARCH") {
        Ok(s) => s,
        Err(_) => String::new(),
    };
    let machine = match env::var("MACHINE") {
        Ok(s) => s,
        Err(_) => String::from("unknown"),
    };
    if !arch_machine_supported(&arch, &machine) {
        panic!(
            "Machine {} not supported on Unishyper for Arch {} now",
            machine, arch
        )
    }
    // set envs
    let build_time = chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S %Z");
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
    let hostname = gethostname::gethostname();
    println!(
        "cargo:rustc-env=HOSTNAME={}",
        hostname.into_string().unwrap()
    );
    println!("cargo:rustc-env=ARCH={}", arch);
    println!("cargo:rustc-env=MACHINE={}", machine);
    built::write_built_file().expect("Failed to acquire build-time information");
    Ok(())
}
