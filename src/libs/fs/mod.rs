mod fat;
pub mod fs;
pub mod interface;

use alloc::boxed::Box;

pub const FAT_ROOT: &str = "/fatfs/";

pub fn init() {
    let root_path = "fatfs";
    fs::FILESYSTEM
        .lock()
        .mount(root_path, Box::new(fat::Fatfs::singleton()))
        .expect("Mount failed!!!");

    info!("fs init success.");
}

use interface::*;

pub fn unlink(path: &str) -> i32 {
    debug!("unlink {}", path);

    fs::FILESYSTEM
        .lock()
        .unlink(path)
        .expect("Unlinking failed!"); // TODO: error handling
    0
}

pub fn open(path: &str, flags: i32, mode: i32) -> i32 {
    debug!("Open {}, {}, {}", path, flags, mode);
    let mut fs = fs::FILESYSTEM.lock();

    let fd = fs.open(path, open_flags_to_perm(flags, mode as u32));
    if let Ok(fd) = fd {
        fd as i32
    } else {
        -1
    }
}

pub fn close(fd: i32) -> i32 {
    // we don't have to close standard descriptors
    if fd < 3 {
        return 0;
    }

    let mut fs = fs::FILESYSTEM.lock();
    fs.close(fd as u64);
    0
}

pub fn read(fd: i32, buf: *mut u8, len: usize) -> isize {
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

pub fn write(fd: i32, buf: *const u8, len: usize) -> isize {
    assert!(len <= isize::MAX as usize);
    let buf = unsafe { core::slice::from_raw_parts(buf, len) };

    if fd > 2 {
        // Normal file
        let mut written_bytes = 0;
        let mut fs = fs::FILESYSTEM.lock();
        fs.fd_op(fd as u64, |file: &mut Box<dyn PosixFile + Send>| {
            written_bytes = file.write(buf).unwrap(); // TODO: might fail
        });
        debug!("Write done! {}", written_bytes);
        written_bytes as isize
    } else {
        unimplemented!("try to write to stdin/err/out");
    }
}

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
fn stat(_file: *const u8, _st: usize) -> i32 {
    unimplemented!("stat is unimplemented");
}

pub fn print_dir(path: &str) -> Result<(), FileError> {
    let fs = fs::FILESYSTEM.lock();
    fs.print_dir(path)
}

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
