use std::{env, error::Error, fs::File, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    // build directory for this crate
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // extend the library search path
    println!("cargo:rustc-link-search={}", out_dir.display());

    let arch = match env::var("ARCH") {
        Ok(s) => s,
        Err(_) => String::new(),
    };
    let machine = match env::var("MACHINE") {
        Ok(s) => s,
        Err(_) => String::from("unknown"),
    };
    match arch.as_str() {
        "x86_64" => match machine.as_str() {
            "qemu" => File::create(out_dir.join("linkerx86.ld"))?
                .write_all(include_bytes!("cfg/x86_64linker.ld"))?,
            _ => panic!(
                "Machine {} not supported on Unishyper for Arch {} now",
                machine, arch
            ),
        },
        "riscv64" => match machine.as_str() {
            "k210" => File::create(out_dir.join("linkerriscv-k210.ld"))?
                .write_all(include_bytes!("cfg/riscv64linker-k210.ld"))?,
            "qemu" => File::create(out_dir.join("linkerriscv.ld"))?
                .write_all(include_bytes!("cfg/riscv64linker.ld"))?,
            _ => panic!(
                "Machine {} not supported on Unishyper for Arch {} now",
                machine, arch
            ),
        },

        // By default, we set arch as aarch64.
        "aarch64" => match machine.as_str() {
            "tx2" => File::create(out_dir.join("linker-tx2.ld"))?
                .write_all(include_bytes!("cfg/linker-tx2.ld"))?,
            "qemu" | "shyper" => File::create(out_dir.join("linker.ld"))?
                .write_all(include_bytes!("cfg/linker.ld"))?,
            _ => panic!(
                "Machine {} not supported on Unishyper for Arch {} now",
                machine, arch
            ),
        },
        _ => panic!(
            "Machine {} not supported on Unishyper for Arch {} now",
            machine, arch
        ),
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
