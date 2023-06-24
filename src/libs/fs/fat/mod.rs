use core::cell::RefCell;

use alloc::{boxed::Box, collections::BTreeMap};
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};

use spin::Once;
use fatfs::{
    Read, Write, Seek, SeekFrom, FileSystem, DefaultTimeProvider, LossyOemCpConverter, FsOptions,
    File,
};

use crate::libs::fs::fat::diskcursor::DiskCursor;

use super::interface::{
    PosixFile, PosixFileSystemInner, FilePerms, FileError, SeekWhence, PosixFileSystem,
};

mod diskcursor;
mod io;

type InnerFatfs = FileSystem<DiskCursor, DefaultTimeProvider, LossyOemCpConverter>;

pub struct Fatfs {
    fs: InnerFatfs,
    fd2file: RefCell<
        BTreeMap<usize, File<'static, DiskCursor, DefaultTimeProvider, LossyOemCpConverter>>,
    >,
}

unsafe impl Sync for Fatfs {}

/// SAFETY: only access in a thread
unsafe impl Send for Fatfs {}

static FATFS: Once<Fatfs> = Once::new();

impl Fatfs {
    fn new() -> Self {
        let fs = FileSystem::new(DiskCursor::new(0), FsOptions::new()).expect("FATFS init failed");
        let fd2file = RefCell::new(BTreeMap::new());
        debug!("fat fs init success.");
        Fatfs { fs, fd2file }
    }
    pub fn singleton() -> &'static Self {
        FATFS.call_once(|| Fatfs::new());
        FATFS.get().unwrap()
    }
}

impl PosixFileSystem for &Fatfs {
    fn open(
        &self,
        path: &str,
        perms: FilePerms,
        fd: usize,
    ) -> Result<Box<dyn PosixFile + Send>, FileError> {
        let fs = &Fatfs::singleton().fs;
        let root = fs.root_dir();
        let res = root.open_file(path);
        match res {
            Ok(file) => {
                debug!("open file on path {}, assigned fd {}", path, fd);
                self.fd2file.borrow_mut().insert(fd, file);
                let my_file = FatfsFile::new(fd, path);
                Ok(Box::new(my_file))
            }
            _ => {
                if !perms.creat {
                    warn!("open file on path {}, file not exist", path);
                    Err(FileError::ENOENT)
                } else {
                    debug!("create file on path {}, assigned fd {}", path, fd);
                    match root.create_file(path) {
                        Ok(file) => {
                            self.fd2file.borrow_mut().insert(fd, file);
                            let my_file = FatfsFile::new(fd, path);
                            Ok(Box::new(my_file))
                        }
                        _ => Err(FileError::EOTHERS),
                    }
                }
            }
        }
    }

    fn unlink(&self, path: &str) -> Result<(), FileError> {
        let fs = &self.fs;
        let root = fs.root_dir();
        match root.remove(path) {
            Ok(_) => Ok(()),
            Err(_) => Err(FileError::ENOENT),
        }
    }

    fn print_dir(&self, path: &str) -> Result<(), FileError> {
        let dir = if path.is_empty() {
            Fatfs::singleton().fs.root_dir()
        } else {
            let root_cursor = &Fatfs::singleton().fs.root_dir();
            let res = root_cursor.open_dir(path);
            match res {
                Ok(dir) => dir,
                _ => {
                    warn!("directory {} not exist in this fat file system", path);
                    return Err(FileError::ENOENT);
                }
            }
        };
        println!("[  ]:[T] [size]\t[name]");
        // ls dir
        for (idx, entry) in dir.iter().enumerate() {
            let entry = entry.expect("Entry");
            println!(
                "[{:>2}]:[{}] {:>5}\t{}",
                idx,
                if entry.is_dir() { "d" } else { "-" },
                entry.len(),
                entry.file_name(),
            );
        }
        Ok(())
    }

    fn create_dir(&self, path: &str) -> Result<(), FileError> {
        debug!("create_dir on path {}", path);
        let fs = &self.fs;
        let root = fs.root_dir();
        match root.create_dir(path) {
            Ok(_) => Ok(()),
            Err(_) => Err(FileError::EOTHERS),
        }
    }
}

#[allow(dead_code)]
impl PosixFileSystemInner for &Fatfs {
    fn close(&self, fd: usize) -> Result<(), FileError> {
        match self.fd2file.borrow_mut().remove(&fd) {
            Some(file) => {
                drop(file);
                Ok(())
            }
            None => Err(FileError::ENOENT),
        }
    }

    fn read(&self, fd: usize, len: u32) -> Result<Vec<u8>, FileError> {
        match self.fd2file.borrow_mut().get_mut(&fd) {
            Some(file) => {
                let mut buf: Vec<u8> = vec![0; len as usize];
                match file.read(&mut buf) {
                    Ok(count) => {
                        buf.truncate(count);
                        Ok(buf)
                    }
                    Err(_) => Err(FileError::EOTHERS),
                }
            }
            None => Err(FileError::ENOENT),
        }
    }

    fn write(&self, fd: usize, buf: &[u8]) -> Result<u64, FileError> {
        match self.fd2file.borrow_mut().get_mut(&fd) {
            Some(file) => match file.write(buf) {
                Ok(res) => Ok(res as u64),
                Err(_) => Err(FileError::EOTHERS),
            },
            None => Err(FileError::ENOENT),
        }
    }

    fn lseek(&self, fd: usize, offset: isize, whence: SeekWhence) -> Result<usize, FileError> {
        match self.fd2file.borrow_mut().get_mut(&fd) {
            Some(file) => {
                let pos = match whence {
                    SeekWhence::Set => SeekFrom::Start(offset as u64),
                    SeekWhence::Cur => SeekFrom::Current(offset as i64),
                    SeekWhence::End => SeekFrom::End(offset as i64),
                };
                match file.seek(pos) {
                    Ok(res) => Ok(res as usize),
                    _ => Err(FileError::EOTHERS),
                }
            }
            None => Err(FileError::ENOENT),
        }
    }
}

#[allow(dead_code)]
struct FatfsFile {
    fd: usize,
    name: String,
}

impl FatfsFile {
    pub fn new(fd: usize, name: &str) -> Self {
        FatfsFile {
            fd,
            name: name.to_string(),
        }
    }
}

impl PosixFile for FatfsFile {
    fn close(&self) -> Result<(), FileError> {
        Fatfs::singleton().close(self.fd)
    }

    fn read(&self, len: u32) -> Result<Vec<u8>, FileError> {
        Fatfs::singleton().read(self.fd, len)
    }

    fn write(&self, buf: &[u8]) -> Result<u64, FileError> {
        Fatfs::singleton().write(self.fd, buf)
    }

    fn lseek(&self, offset: isize, whence: SeekWhence) -> Result<usize, FileError> {
        Fatfs::singleton().lseek(self.fd, offset, whence)
    }
}
