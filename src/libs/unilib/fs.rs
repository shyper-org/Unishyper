//! Unilib-FS
//! Filesystem manipulation operations of unilib.
//!
//! This module contains basic methods to manipulate the contents of the unilib
//! filesystem. All methods in this module just trigger a HVC request to hypervisor,
//! which send a request to MVM to use its origin file system to finish to operation.

use crate::hvc_mode;
use crate::libs::unilib::hvc::*;
use crate::libs::fs::interface::{
    O_RDONLY, O_WRONLY, O_RDWR, O_CREAT, O_EXCL, O_TRUNC, O_APPEND, O_DIRECT,
};
use crate::mm::address::*;

pub fn init() {
    hvc_call(0, 0, 0, hvc_mode!(HVC_UNILIB, HVC_UNILIB_FS_INIT));
}

/// **Open** API for unilib fs, attempts to open a file at given path of expected flags and mode.
/// HVC_UNILIB | HVC_UNILIB_FS_OPEN
/// Returns the file descriptor of the newly opened file, or -1 on failure.
/// ## Arguments
/// * `path`    - The intermediated physical address of the path that GVM wants to open through unilib-fs API.
/// * `flags`   - The flags of open API, we need to care about the transfer between C and Rust.
///                Currently support O_RDONLY | O_WRONLY | O_RDWR | O_CREAT | O_EXCL | O_TRUNC | O_APPEND | O_DIRECT)
/// * `mode`    - The mode of open API, currently unsurpported .
pub fn open(path: &str, flags: i32, mode: i32) -> i32 {
    debug!("Open {}, {}, {}", path, flags, mode);

    // Check flags.
    if flags & !(O_RDONLY | O_WRONLY | O_RDWR | O_CREAT | O_EXCL | O_TRUNC | O_APPEND | O_DIRECT)
        != 0
    {
        warn!("Unknown file flags used! {}", flags);
        return - 1;
    }

    let path_va = &path.as_bytes()[0] as *const _ as usize;
    let path_pa = VAddr::from(path_va).to_physical_address().value();
    debug!(
        "path address 0x{:p} , convert to 0x{:x}, get pa 0x{:x}",
        path, path_va, path_pa
    );
    let fd = hvc_call(
        path_pa,
        path.len(),
        flags as usize,
        hvc_mode!(HVC_UNILIB, HVC_UNILIB_FS_OPEN),
    ) as i32;
    debug!("Open get fd {}", fd);
    fd
}

/// **Close** API for unilib fs, attempts to close a file by given fd.
/// HVC_UNILIB | HVC_UNILIB_FS_CLOSE
/// Returns the operation result passed from MVM's shyper-cli, or -1 on failure.
/// ## Arguments
/// * `fd`  - The file descriptor of file to be closed.
pub fn close(fd: i32) -> i32 {
    // we don't have to close standard descriptors
    debug!("close fd {}", fd);

    if fd < 3 {
        unimplemented!("try to close stdin/err/out");
    }

    let res = hvc_call(
        fd as usize,
        0,
        0,
        hvc_mode!(HVC_UNILIB, HVC_UNILIB_FS_CLOSE),
    ) as i32;
    debug!("close fd {} get res {}", fd, res);
    res
}

/// **Read** API for unilib fs.
/// HVC_UNILIB | HVC_UNILIB_FS_READ
/// Read NBYTES into BUF from FD.
/// If success, returns the number read, -1 for errors or 0 for EOF.
/// ## Arguments
/// * `fd`  - The file descriptor of file to read.
/// * `buf` - The buffer to be read into.
/// * `len` - Number of bytes to be read.
pub fn read(fd: i32, buf: *mut u8, len: usize) -> isize {
    debug!("Read! {}, {}", fd, len);

    if fd < 3 {
        unimplemented!("try to read stdin/err/out");
    }

    let buf_pa = VAddr::from(buf as usize).to_physical_address().value();
    debug!(
        "read buffer address 0x{:x} , convert to pa 0x{:x}",
        buf as usize, buf_pa
    );

    let read_bytes = hvc_call(
        fd as usize,
        buf_pa,
        len,
        hvc_mode!(HVC_UNILIB, HVC_UNILIB_FS_READ),
    );

    debug!("read success, read_bytes {}", read_bytes);

    read_bytes as isize
}

/// **Write** API for unilib fs.
/// HVC_UNILIB | HVC_UNILIB_FS_WRITE
/// This function performs the write operation by send a HvcGuestMsg to MVM.
/// Write N bytes of BUF to FD. Return the number written.
/// It's a synchronous process trigger by GVM.
/// If success, returns the number written, or -1, wrapped by `Result` structure.
/// ## Arguments
/// * `fd`  - The file descriptor of file to write to.
/// * `buf` - The buffer waiting to be written to the target file.
/// * `len` - Number of bytes to be written.
pub fn write(fd: i32, buf: *const u8, len: usize) -> isize {
    debug!("Write! {}, {}", fd, len);

    if fd < 3 {
        unimplemented!("try to write to stdin/err/out");
    }

    assert!(len <= isize::MAX as usize);
    let buf_pa = VAddr::from(buf as usize).to_physical_address().value();
    debug!(
        "write buffer address 0x{:x} , convert to pa 0x{:x}",
        buf as usize, buf_pa
    );

    let written_bytes = hvc_call(
        fd as usize,
        buf_pa,
        len,
        hvc_mode!(HVC_UNILIB, HVC_UNILIB_FS_WRITE),
    );

    debug!("write success, read_bytes {}", written_bytes);
    written_bytes as isize
}

/// **Lseek** API for unilib fs.
/// HVC_UNILIB | HVC_UNILIB_FS_LSEEK
/// Reposition read/write file offset.
/// lseek() repositions the file offset of the open file description associated with the file descriptor fd to the argument offset according to the directive whence.
/// Upon successful completion, lseek() returns the resulting offset location as measured in bytes from the beginning of the file.
/// ## Arguments
/// * `fd`     - The file descriptor of file.
/// * `offset` - The file offset of the open file.
/// * `whence` - Only can be these three following types currently:
///                 SEEK_SET (0) : Seek from beginning of file, the file offset is set to offset bytes.
///                 SEEK_CUR (1) : Seek from current position, the file offset is set to its current location plus offset bytes.
///                 SEEK_END (2) : Seek from end of file, the file offset is set to the size of the file plus offset bytes.
pub fn lseek(fd: i32, offset: isize, whence: i32) -> isize {
    debug!("lseek! fd {}, offset {}, whence {}", fd, offset, whence);

    if fd < 3 {
        unimplemented!("try to write to stdin/err/out");
    }

    let ret = hvc_call(
        fd as usize,
        offset as usize,
        whence as usize,
        hvc_mode!(HVC_UNILIB, HVC_UNILIB_FS_LSEEK),
    );

    debug!("lseek fd {} get res {}", fd, ret);

    ret as isize
}

/// **Stat** API for unilib fs.
/// HVC_UNILIB | HVC_UNILIB_FS_STAT
/// Currently unsupported.
///
/// Given a path, query the file system to get information about a file,
/// directory, etc.
///
/// This function will traverse symbolic links to query information about the
/// destination file.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `stat` function on Unix
/// and the `GetFileInformationByHandle` function on Windows.
#[allow(unused)]
fn stat(_file: *const u8, _st: usize) -> i32 {
    let _ = hvc_call(0, 0, 0, hvc_mode!(HVC_UNILIB, HVC_UNILIB_FS_STAT));
    unimplemented!("stat is unimplemented");
}
