/// Shyper unikernel abi for fs operations.
/// Stdin/out/err operations may use some functions with fd < 3. 
use alloc::format;

use crate::libs::fs;


#[no_mangle]
pub extern "C" fn shyper_open(name: *const u8, flags: i32, mode: i32) -> i32 {
    let path = unsafe { core::ffi::CStr::from_ptr(name as _) }.to_str().unwrap();
	fs::open(format!("{}{}", fs::FS_ROOT, path).as_str(), flags, mode)
}

#[no_mangle]
pub extern "C" fn shyper_read(fd: i32, buf: *mut u8, len: usize) -> isize {
    if fd > 2 {
        // Normal file
        if cfg!(feature = "fs") {
            fs::read(fd, buf, len)
        } else {
            warn!(
                "\"fs\" feature is not enabled for shyper, read from fd {} failed",
                fd
            );
            0 as isize
        }
    } else {
        warn!("read from stdin/err/out is unimplemented, returning -1");
        -1 as isize
    }
}

#[no_mangle]
pub extern "C" fn shyper_write(fd: i32, buf: *const u8, len: usize) -> isize {
    if fd > 2 {
        // Normal file
        if cfg!(feature = "fs") {
            fs::write(fd, buf, len)
        } else {
            warn!(
                "\"fs\" feature is not enabled for shyper, write to fd {} failed",
                fd
            );
            0 as isize
        }
    } else {
        // stdin/err/out all go to console
        let buf = unsafe { core::slice::from_raw_parts(buf, len) };
        crate::libs::print::print_byte(buf);
        len as isize
    }
}

#[no_mangle]
pub extern "C" fn shyper_lseek(fd: i32, offset: isize, whence: i32) -> isize {
    if fd > 2 {
        // Normal file
        if cfg!(feature = "fs") {
            fs::lseek(fd, offset, whence)
        } else {
            warn!(
                "\"fs\" feature is not enabled for shyper, seek fd {} failed",
                fd
            );
            0 as isize
        }
    } else {
        warn!("seek stdin/err/out is unimplemented, returning -1");
        -1 as isize
    }
}

#[no_mangle]
pub extern "C" fn shyper_close(fd: i32) -> i32 {
    if fd > 2 {
        // Normal file
        if cfg!(feature = "fs") {
            fs::close(fd)
        } else {
            warn!(
                "\"fs\" feature is not enabled for shyper, close fd {} failed",
                fd
            );
            0 as i32
        }
    } else {
        // we don't have to close standard descriptors
        0 as i32
    }
}

#[no_mangle]
pub extern "C" fn shyper_unlink(name: *const i8) -> i32 {
    let path = unsafe { core::ffi::CStr::from_ptr(name as _) }.to_str().unwrap();
    fs::unlink(format!("{}{}", fs::FS_ROOT, path).as_str())
}


#[no_mangle]
pub extern "C" fn shyper_stat(file: *const u8, st: usize) -> i32 {
    fs::stat(file, st)
}