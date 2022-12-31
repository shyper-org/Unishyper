/// Shyper unikernel abi for fs operations.
/// Stdin/out/err operations may use some functions with fd < 3.

#[cfg(feature = "fs")]
use crate::libs::fs;

#[cfg(not(feature = "fs"))]
#[no_mangle]
pub extern "C" fn shyper_open(_name: *const u8, _flags: i32, _mode: i32) -> i32 {
    warn!("\"fs\" feature is not enabled for shyper, open file failed");
    0 as i32
}

#[cfg(feature = "fs")]
#[no_mangle]
pub extern "C" fn shyper_open(name: *const u8, flags: i32, mode: i32) -> i32 {
    use alloc::format;
    let path = unsafe { core::ffi::CStr::from_ptr(name as _) }
        .to_str()
        .unwrap();
    fs::open(format!("{}{}", fs::FS_ROOT, path).as_str(), flags, mode)
}

#[cfg(not(feature = "fs"))]
#[no_mangle]
pub extern "C" fn shyper_read(fd: i32, _buf: *mut u8, _len: usize) -> isize {
    if fd > 2 {
        warn!(
            "\"fs\" feature is not enabled for shyper, read from fd {} failed",
            fd
        );
        0 as isize
    } else {
        warn!("read from stdin/err/out is unimplemented, returning -1");
        -1 as isize
    }
}

#[cfg(feature = "fs")]
#[no_mangle]
pub extern "C" fn shyper_read(fd: i32, buf: *mut u8, len: usize) -> isize {
    if fd > 2 {
        // Normal file
        fs::read(fd, buf, len)
    } else {
        warn!("read from stdin/err/out is unimplemented, returning -1");
        -1 as isize
    }
}

#[cfg(not(feature = "fs"))]
#[no_mangle]
pub extern "C" fn shyper_write(fd: i32, buf: *const u8, len: usize) -> isize {
    if fd > 2 {
        warn!(
            "\"fs\" feature is not enabled for shyper, write to fd {} failed",
            fd
        );
        0 as isize
    } else {
        // stdin/err/out all go to console
        let buf = unsafe { core::slice::from_raw_parts(buf, len) };
        crate::libs::print::print_byte(buf);
        len as isize
    }
}

#[cfg(feature = "fs")]
#[no_mangle]
pub extern "C" fn shyper_write(fd: i32, buf: *const u8, len: usize) -> isize {
    if fd > 2 {
        // Normal file
        fs::write(fd, buf, len)
    } else {
        // stdin/err/out all go to console
        let buf = unsafe { core::slice::from_raw_parts(buf, len) };
        crate::libs::print::print_byte(buf);
        len as isize
    }
}

#[cfg(not(feature = "fs"))]
#[no_mangle]
pub extern "C" fn shyper_lseek(fd: i32, _offset: isize, _whence: i32) -> isize {
    if fd > 2 {
        warn!(
            "\"fs\" feature is not enabled for shyper, seek fd {} failed",
            fd
        );
        0 as isize
    } else {
        warn!("seek stdin/err/out is unimplemented, returning -1");
        -1 as isize
    }
}

#[cfg(feature = "fs")]
#[no_mangle]
pub extern "C" fn shyper_lseek(fd: i32, offset: isize, whence: i32) -> isize {
    if fd > 2 {
        // Normal file
        fs::lseek(fd, offset, whence)
    } else {
        warn!("seek stdin/err/out is unimplemented, returning -1");
        -1 as isize
    }
}

#[cfg(not(feature = "fs"))]
#[no_mangle]
pub extern "C" fn shyper_close(fd: i32) -> i32 {
    if fd > 2 {
        warn!(
            "\"fs\" feature is not enabled for shyper, close fd {} failed",
            fd
        );
        0 as i32
    } else {
        // we don't have to close standard descriptors
        0 as i32
    }
}

#[cfg(feature = "fs")]
#[no_mangle]
pub extern "C" fn shyper_close(fd: i32) -> i32 {
    if fd > 2 {
        // Normal file
        fs::close(fd)
    } else {
        // we don't have to close standard descriptors
        0 as i32
    }
}

#[cfg(not(feature = "fs"))]
#[no_mangle]
pub extern "C" fn shyper_unlink(_name: *const i8) -> i32 {
    warn!("\"fs\" feature is not enabled for shyper, unlink file failed");
    0 as i32
}

#[cfg(feature = "fs")]
#[no_mangle]
pub extern "C" fn shyper_unlink(name: *const i8) -> i32 {
    use alloc::format;
    let path = unsafe { core::ffi::CStr::from_ptr(name as _) }
        .to_str()
        .unwrap();
    fs::unlink(format!("{}{}", fs::FS_ROOT, path).as_str())
}

#[cfg(not(feature = "fs"))]
#[no_mangle]
pub extern "C" fn shyper_stat(_file: *const u8, _st: usize) -> i32 {
    warn!("\"fs\" feature is not enabled for shyper, stat file failed");
    0 as i32
}

#[cfg(feature = "fs")]
#[no_mangle]
pub extern "C" fn shyper_stat(file: *const u8, st: usize) -> i32 {
    fs::stat(file, st)
}
