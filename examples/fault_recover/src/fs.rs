use unishyper::*;
use unishyper::fs;
use unishyper::io;
use unishyper::fs::{File, Path};

use alloc::string::String;

pub extern "C" fn test_fs(_arg: usize) {
    println!("[TEST] === test fs operation ===");

    let mut f = File::open(&Path::new("/fatfs/test.txt")).unwrap();

    println!("[TEST] === open success");

    let s = "testtest";
    f.write_all(s.as_bytes());

    println!("[TEST] === write success");

    let mut str = vec![0 as u8; 10];
    match f.read(&mut str) {
        Ok(_) => {
            let s = String::from_utf8(str).expect("failed to convert vec u8 to string");
        }
        Err(e) => panic!("read failed"),
    }

    println!("[TEST] === read success");

    drop(f);
}
