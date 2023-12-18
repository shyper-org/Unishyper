use alloc::{boxed::Box, vec::Vec};

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum FileError {
    // No such file or directory.
    ENOENT,
    EOTHERS,
}

pub const O_RDONLY: i32 = 0o0000;
pub const O_WRONLY: i32 = 0o0001;
pub const O_RDWR: i32 = 0o0002;
pub const O_CREAT: i32 = 0o0100;
pub const O_EXCL: i32 = 0o0200;
pub const O_TRUNC: i32 = 0o1000;
pub const O_APPEND: i32 = 0o2000;
pub const O_DIRECT: i32 = 0o40000;

#[allow(dead_code)]
// Pending: raw is partially redundant, create nicer interface
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

/// Currently the mode bit is unsupported.
pub fn open_flags_to_perm(flags: i32, mode: u32) -> FilePerms {
    let mut perms = FilePerms {
        raw: flags as u32,
        mode,
        ..Default::default()
    };
    perms.write = flags & (O_WRONLY | O_RDWR) != 0;
    perms.creat = flags & (O_CREAT) != 0;
    perms.excl = flags & (O_EXCL) != 0;
    perms.trunc = flags & (O_TRUNC) != 0;
    perms.append = flags & (O_APPEND) != 0;
    perms.directio = flags & (O_DIRECT) != 0;
    if flags & !(O_RDONLY | O_WRONLY | O_RDWR | O_CREAT | O_EXCL | O_TRUNC | O_APPEND | O_DIRECT)
        != 0
    {
        warn!("Unknown file flags used! {}", flags);
    }
    perms
}

#[allow(dead_code)]
pub enum SeekWhence {
    Set,
    Cur,
    End,
}

const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;
const SEEK_END: i32 = 2;

#[allow(unused_variables)]
#[allow(unreachable_patterns)]
impl TryFrom<i32> for SeekWhence {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            SEEK_CUR => Ok(SeekWhence::Cur),
            SEEK_SET => Ok(SeekWhence::Set),
            SEEK_END => Ok(SeekWhence::End),
            _ => Err("Got invalid seek whence parameter!"),
        }
    }
}

#[allow(unused_variables)]
#[allow(unreachable_patterns)]
impl TryInto<i32> for SeekWhence {
    type Error = &'static str;
    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            SeekWhence::Cur => Ok(SEEK_CUR),
            SeekWhence::Set => Ok(SEEK_SET),
            SeekWhence::End => Ok(SEEK_END),
            _ => Err("Got invalid seek whence parameter!"),
        }
    }
}

pub trait PosixFileSystem {
    fn open(
        &self,
        path: &str,
        perms: FilePerms,
        fd: usize,
    ) -> Result<Box<dyn PosixFile + Send>, FileError>;
    fn unlink(&self, path: &str) -> Result<(), FileError>;
    fn print_dir(&self, path: &str) -> Result<(), FileError>;
    fn create_dir(&self, path: &str) -> Result<(), FileError>;
}

pub trait PosixFile {
    fn close(&self) -> Result<(), FileError>;
    fn read(&self, len: u32) -> Result<Vec<u8>, FileError>;
    fn write(&self, buf: &[u8]) -> Result<u64, FileError>;
    fn lseek(&self, offset: isize, whence: SeekWhence) -> Result<usize, FileError>;
}

pub trait PosixFileSystemInner {
    fn close(&self, fd: usize) -> Result<(), FileError>;
    fn read(&self, fd: usize, len: u32) -> Result<Vec<u8>, FileError>;
    fn write(&self, fd: usize, buf: &[u8]) -> Result<u64, FileError>;
    fn lseek(&self, fd: usize, offset: isize, whence: SeekWhence) -> Result<usize, FileError>;
}

#[derive(Debug)]
pub struct AtaError;

pub trait BlkIO {
    fn read(&mut self, sector: usize, count: usize) -> Result<(), AtaError>;
    fn write(&self, sector: usize, count: usize) -> Result<(), AtaError>;
    fn get_data(&self, offset: usize) -> &[u8];
    fn get_data_mut(&mut self, offset: usize) -> &mut [u8];
}
