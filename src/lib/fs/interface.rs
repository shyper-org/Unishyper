use alloc::{boxed::Box, vec::Vec};

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum FileError {
    ENOENT,
    EOTHERS,
}

#[allow(dead_code)]
// TODO: raw is partially redundant, create nicer interface
#[derive(Clone, Copy, Debug, Default)]
pub struct FilePerms {
    pub write: bool,
    pub creat: bool,
    pub excl: bool,
    pub trunc: bool,
    pub append: bool,
    pub directio: bool,
    pub raw: u32,
    pub mode: u32,
}

#[allow(dead_code)]
pub enum SeekWhence {
    Set,
    Cur,
    End,
}

pub trait PosixFileSystem {
    fn open(&mut self, _path: &str, _perms: FilePerms) -> Result<Box<dyn PosixFile>, FileError>;
    fn unlink(&mut self, _path: &str) -> Result<(), FileError>;
}

pub trait PosixFile {
    fn close(&mut self) -> Result<(), FileError>;
    fn read(&mut self, len: u32) -> Result<Vec<u8>, FileError>;
    fn write(&mut self, buf: &[u8]) -> Result<u64, FileError>;
    fn lseek(&mut self, offset: isize, whence: SeekWhence) -> Result<usize, FileError>;
}

pub trait PosixFileSystemInner {
    fn close(&mut self, fd: usize) -> Result<(), FileError>;
    fn read(&mut self, fd: usize, len: u32) -> Result<Vec<u8>, FileError>;
    fn write(&mut self, fd: usize, buf: &[u8]) -> Result<u64, FileError>;
    fn lseek(&mut self, fd: usize, offset: isize, whence: SeekWhence) -> Result<usize, FileError>;
}

#[derive(Debug)]
pub struct AtaError;

pub trait BlkIO {
    fn read(&self, sector: usize, count: usize) -> Result<(), AtaError>;
    fn write(&self, sector: usize, count: usize) -> Result<(), AtaError>;
    fn get_data(&self, offset: usize) -> &[u8];
    fn get_data_mut(&mut self, offset: usize) -> &mut [u8];
}
