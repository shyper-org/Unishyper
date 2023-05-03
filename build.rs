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
    match arch.as_str() {
        "x86_64" => File::create(out_dir.join("linkerx86.ld"))?
            .write_all(include_bytes!("cfg/x86_64linker.ld"))?,
        // By default, we set arch as aarch64.
        _ => {
            let machine = match env::var("MACHINE") {
                Ok(s) => s,
                Err(_) => String::new(),
            };
            match machine.as_str() {
                "tx2" => File::create(out_dir.join("linker-tx2.ld"))?
                    .write_all(include_bytes!("cfg/linker-tx2.ld"))?,
                _ => File::create(out_dir.join("linker.ld"))?
                    .write_all(include_bytes!("cfg/linker.ld"))?,
            }
        }
    }

    Ok(())
}
