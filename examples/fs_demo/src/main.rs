#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![allow(unused_imports)]

use rust_shyper_os::*;
use rust_shyper_os::fs;
use rust_shyper_os::io;
use rust_shyper_os::fs::{File,Path};

use alloc::string::String;

#[macro_use]
extern crate alloc;

/// Simple implementation for `% cat path`
fn cat(path: &Path) -> io::Result<String> {
	let f = File::open(path)?;
	let mut str = vec![0 as u8; 10];
	match f.read(&mut str) {
		Ok(_) => {
            let s = String::from_utf8(str).expect("failed to convert vec u8 to string");
            Ok(s)
        },
		Err(e) => Err(e),
	}
}

/// Simple implementation for `% echo s > path`
fn echo(s: &str, path: &Path) -> io::Result<()> {
	let mut f = File::create(path)?;
	f.write_all(s.as_bytes())
}

/// Simple implementation for `% touch path`
fn touch(path: &Path) -> io::Result<()> {
	match File::create(path) {
		Ok(_) => Ok(()),
		Err(e) => Err(e),
	}
}


#[no_mangle]
fn main() {
	println!("`echo hello > /fatfs/echo.txt`");
    echo("hello", &Path::new("/fatfs/echo.txt")).unwrap_or_else(|why| {
		println!("! {:?}", why);
	});

	println!("`touch /fatfs/touch.txt`");
    touch(&Path::new("/fatfs/touch.txt")).unwrap_or_else(|why| {
		println!("! {:?}", why);
	});

    println!("`cat /fatfs/echo.txt`");
	match cat(&Path::new("/fatfs/echo.txt")) {
		Err(why) => println!("! {:?}", why),
		Ok(s) => println!("> {}", s),
	}
    
    exit();
}
