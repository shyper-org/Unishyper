#[cfg(feature = "fat")]
mod fat;

#[cfg(feature = "unilib")]
mod unilib;

pub mod fs;
pub mod interface;

use alloc::boxed::Box;

#[cfg(feature = "fat")]
pub const FAT_FS_ROOT: &str = "/fatfs/";

#[cfg(feature = "unilib")]
pub const UNILIB_FS_ROOT: &str = "/unilibfs/";

#[cfg(all(not(feature = "fat"), not(feature = "unilib")))]
compile_error!("When \"fs\" feature is enabled, you need to choose fs type, which means at least one of the  features \"fat\" and \"unilib\" should be enabled.");

/// By default, the terminal's fs operation is operated under FAT-fs's directory.
// #[cfg(not(feature = "unilib"))]
#[cfg(all(feature = "fat", not(feature = "unilib")))]
pub const FS_ROOT: &str = FAT_FS_ROOT;

#[cfg(all(feature = "unilib", not(feature = "fat")))]
pub const FS_ROOT: &str = UNILIB_FS_ROOT;

#[cfg(all(feature = "fat", feature = "unilib"))]
pub const FS_ROOT: &str = "";

pub fn init() {
    #[cfg(feature = "fat")]
    {
        fs::FILESYSTEM
            .lock()
            .mount("fatfs", Box::new(fat::Fatfs::singleton()))
            .expect("Mount failed!!!");
        info!("fat fs mount success on \"{}\".", FAT_FS_ROOT);
    }

    #[cfg(feature = "unilib")]
    {
        fs::FILESYSTEM
            .lock()
            .mount("unilibfs", Box::new(unilib::UnilibFs::new()))
            .expect("Mount failed!!!");
        info!("unilib fs mount success on \"{}\".", UNILIB_FS_ROOT);
    }
    info!("fs init success.");

    // This is just for uni-lib test during development.
    if false {
        use alloc::vec;
        #[cfg(not(feature = "std"))]
        use alloc::{format, string::String};
        // Test open.
        debug!("This is just for uni-lib test during development");
        let fd = open(format!("{}{}", FS_ROOT, "test_path").as_str(), 111, 222);
        debug!("Open test passed, get fd {}", fd);

        // Test write.
        let write_str = "hello";
        let write_res = write(fd, write_str.as_bytes().as_ptr(), write_str.len());
        debug!("Write test passed, get write_res {}", write_res);

        // Test lseek.
        // SEEK_SET = 0
        let lseek_res = lseek(fd, 0, 0);
        debug!("Lseek test passed, get lseek_res {}", lseek_res);

        // Test read.
        let mut read_str = vec![0 as u8; 10];
        let read_res = read(fd, &mut read_str[0], write_str.len());
        debug!(
            "Read test passed, get res {}, get str '{}'",
            read_res,
            String::from_utf8(read_str).expect("failed to convert vec u8 to string")
        );

        // Test lseek.
        // SEEK_SET = 0
        let lseek_res = lseek(fd, 1, 0);
        debug!("Lseek test passed, get lseek_res {}", lseek_res);

        // After lseek, start should remove to 1;
        let mut read_str = vec![0 as u8; 3];
        let read_res = read(fd, &mut read_str[0], 3);
        debug!(
            "After Lseek to 1, read buffer of 3 bytes, get res {}, get str '{}'",
            read_res,
            String::from_utf8(read_str).expect("failed to convert vec u8 to string")
        );

        // Test close.
        let close_res = close(fd);
        debug!("CLose test passed, get close_res {}", close_res);
    }
}

use interface::*;

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn unlink(path: &str) -> i32 {
    debug!("unlink {}", path);

    fs::FILESYSTEM
        .lock()
        .unlink(path)
        .expect("Unlinking failed!"); // TODO: error handling
    0
}

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn open(path: &str, flags: i32, mode: i32) -> i32 {
    debug!("Open {}, {}, {}", path, flags, mode);
    let mut fs = fs::FILESYSTEM.lock();

    let fd = fs.open(path, open_flags_to_perm(flags, mode as u32));
    match fd {
        Ok(fd) => fd as i32,
        Err(err) => {
            warn!("fs open path {} error {:?}", path, err);
            -1
        }
    }
}

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn close(fd: i32) -> i32 {
    assert!(fd > 2);
    let mut fs = fs::FILESYSTEM.lock();
    fs.close(fd as u64);
    0
}

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn read(fd: i32, buf: *mut u8, len: usize) -> isize {
    assert!(len <= isize::MAX as usize);
    assert!(fd > 2);
    debug!("Read! {}, {}", fd, len);

    let mut fs = fs::FILESYSTEM.lock();
    let mut read_bytes = 0;
    fs.fd_op(fd as u64, |file: &mut Box<dyn PosixFile + Send>| {
        let dat = file.read(len as u32).unwrap(); // TODO: might fail
        read_bytes = dat.len();
        unsafe {
            core::slice::from_raw_parts_mut(buf, read_bytes).copy_from_slice(&dat);
        }
    });

    read_bytes as isize
}

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn write(fd: i32, buf: *const u8, len: usize) -> isize {
    assert!(len <= isize::MAX as usize);
    assert!(fd > 2);
    let buf = unsafe { core::slice::from_raw_parts(buf, len) };

    // Normal file
    let mut written_bytes = 0;
    let mut fs = fs::FILESYSTEM.lock();
    fs.fd_op(fd as u64, |file: &mut Box<dyn PosixFile + Send>| {
        written_bytes = file.write(buf).unwrap(); // TODO: might fail
    });
    debug!("Write done! {}", written_bytes);
    written_bytes as isize
}

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn lseek(fd: i32, offset: isize, whence: i32) -> isize {
    debug!("lseek! {}, {}, {}", fd, offset, whence);

    let mut fs = fs::FILESYSTEM.lock();
    let mut ret = 0;
    fs.fd_op(fd as u64, |file: &mut Box<dyn PosixFile + Send>| {
        ret = file.lseek(offset, whence.try_into().unwrap()).unwrap(); // TODO: might fail
    });

    ret as isize
}

#[allow(unused)]
pub fn stat(file: *const u8, st: usize) -> i32 {
    unimplemented!("stat is unimplemented");
}

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn print_dir(path: &str) -> Result<(), FileError> {
    #[cfg(all(feature = "fat", feature = "unilib"))]
    if path == "" {
        println!("[  ]:[T] [size]\t[name]");
        println!("[{:>2}]:[{}] {:>5}\t{}", 0, "d", "-", FAT_FS_ROOT);
        println!("[{:>2}]:[{}] {:>5}\t{}", 1, "d", "-", UNILIB_FS_ROOT);
        return Ok(());
    }

    let fs = fs::FILESYSTEM.lock();
    fs.print_dir(path)
}

#[cfg_attr(feature = "unwind-test", inject::panic_inject, inject::count_stmts)]
pub fn create_dir<P: AsRef<str>>(path: P) -> Result<(), FileError> {
    let fs = fs::FILESYSTEM.lock();
    fs.create_dir(path.as_ref())
}

pub fn remove_file<P: AsRef<str>>(_path: P) -> Result<(), FileError> {
    Ok(())
}

pub fn remove_directory<P: AsRef<str>>(_path: P) -> Result<(), FileError> {
    Ok(())
}
