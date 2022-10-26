use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};
use zerocopy::AsBytes;

use crate::libs::unilib;
use super::interface::{PosixFile, FilePerms, FileError, SeekWhence, PosixFileSystem};

pub struct UnilibFs {}

unsafe impl Sync for UnilibFs {}

/// SAFETY: only access in a thread
unsafe impl Send for UnilibFs {}

impl UnilibFs {
    pub fn new() -> Self {
        unilib::fs::init();
        info!("unilib-fs init success.");
        UnilibFs {}
    }
}

impl PosixFileSystem for UnilibFs {
    fn open(
        &self,
        path: &str,
        _perms: FilePerms,
        _fd: usize,
    ) -> Result<Box<dyn PosixFile + Send>, FileError> {
        let fd = unilib::fs::open(path, 0, 0);
        if fd < 0 {
            return Err(FileError::EOTHERS);
        }
        let my_file = UnilibFile::new(fd as usize, path);
        Ok(Box::new(my_file))
    }

    fn unlink(&self, _path: &str) -> Result<(), FileError> {
        warn!("[warning] unlink not implemented in unilib-fs.");
        Ok(())
    }

    fn print_dir(&self, _path: &str) -> Result<(), FileError> {
        warn!("[warning] print_dir not implemented in unilib-fs.");
        Ok(())
    }

    fn create_dir(&self, _path: &str) -> Result<(), FileError> {
        warn!("[warning] create_dir not implemented in unilib-fs.");
        Ok(())
    }
}

#[allow(dead_code)]
struct UnilibFile {
    fd: usize,
    name: String,
}

impl UnilibFile {
    pub fn new(fd: usize, name: &str) -> Self {
        UnilibFile {
            fd,
            name: name.to_string(),
        }
    }
}

impl PosixFile for UnilibFile {
    fn close(&self) -> Result<(), FileError> {
        let res = unilib::fs::close(self.fd as i32);
        if res < 0 {
            Err(FileError::ENOENT)
        } else {
            Ok(())
        }
    }

    fn read(&self, len: u32) -> Result<Vec<u8>, FileError> {
        let mut buf: Vec<u8> = vec![0; len as usize];
        let read_len = unilib::fs::read(self.fd as i32, &mut buf.as_bytes_mut()[0], len as usize);
        if read_len <= 0 {
            Err(FileError::EOTHERS)
        } else {
            buf.truncate(read_len as usize);
            Ok(buf)
        }
    }

    fn write(&self, buf: &[u8]) -> Result<u64, FileError> {
        let write_len = unilib::fs::write(self.fd as i32, &buf[0], buf.len());
        if write_len <= 0 {
            Err(FileError::EOTHERS)
        } else {
            Ok(write_len as u64)
        }
    }

    fn lseek(&self, offset: isize, whence: SeekWhence) -> Result<usize, FileError> {
        let res = unilib::fs::lseek(self.fd as i32, offset, whence.try_into().unwrap());
        if res < 0 {
            Err(FileError::ENOENT)
        } else {
            Ok(res as usize)
        }
    }
}
