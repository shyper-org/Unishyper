use alloc::sync::Arc;
use alloc::{boxed::Box, collections::BTreeMap};
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};

use spin::{Mutex, RwLock};
use fatfs::{
    IoBase, IoError, Read, Write, Seek, SeekFrom, FileSystem, DefaultTimeProvider,
    LossyOemCpConverter, FsOptions, File,
};

use super::io::{DiskCursor, DiskCursorIoError, BSIZE};

use super::interface::{PosixFile, PosixFileSystemInner, FilePerms, FileError, SeekWhence, BlkIO};

impl IoError for DiskCursorIoError {
    fn is_interrupted(&self) -> bool {
        false
    }

    fn new_unexpected_eof_error() -> Self {
        Self::UnexpectedEof
    }

    fn new_write_zero_error() -> Self {
        Self::WriteZero
    }
}

impl IoBase for DiskCursor {
    type Error = DiskCursorIoError;
}

impl Read for DiskCursor {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DiskCursorIoError> {
        let mut i = 0;
        while i < buf.len() {
            let count = ((buf.len() - i) / BSIZE).max(1);
            let block = self.cache.get(self.sector, count);

            let data = block.get_data(self.offset);
            if data.len() == 0 {
                break;
            }

            let end = (i + data.len()).min(buf.len());
            let len = end - i;
            buf[i..end].copy_from_slice(&data[..len]);

            i += len;
            self.move_cursor(len);
        }
        Ok(i)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DiskCursorIoError> {
        let n = self.read(buf)?;
        assert!(n == buf.len(), "TODO: Error");
        Ok(())
    }
}

impl Write for DiskCursor {
    fn write(&mut self, buf: &[u8]) -> Result<usize, DiskCursorIoError> {
        let mut i = 0;
        while i < buf.len() {
            let count = ((buf.len() - i) / BSIZE).max(1);
            let block = self.cache.get(self.sector, count);

            let data = block.get_data_mut(self.offset);
            if data.len() == 0 {
                break;
            }

            let end = (i + data.len()).min(buf.len());
            let len = end - i;
            data[..end].copy_from_slice(&buf[i..len]);

            block.write(self.sector, count).expect("ata error");

            i += len;
            self.move_cursor(len);
        }
        Ok(i)
    }

    fn flush(&mut self) -> Result<(), DiskCursorIoError> {
        Ok(())
    }
}

impl Seek for DiskCursor {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, DiskCursorIoError> {
        match pos {
            SeekFrom::Start(i) => {
                self.set_position(i as usize);
                Ok(i)
            }
            SeekFrom::End(_i) => {
                unimplemented!()
            }
            SeekFrom::Current(i) => {
                let new_pos = (self.get_position() as i64) + i;
                self.set_position(new_pos as usize);
                Ok(new_pos as u64)
            }
        }
    }
}

struct LockedDiskCursor {
    inner: Mutex<DiskCursor>,
}

impl IoBase for LockedDiskCursor {
    type Error = DiskCursorIoError;
}

impl LockedDiskCursor {
    pub fn new(start_sector: usize) -> Self {
        Self {
            inner: Mutex::new(DiskCursor::new(start_sector)),
        }
    }
}

impl Read for LockedDiskCursor {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DiskCursorIoError> {
        self.inner.lock().read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DiskCursorIoError> {
        self.inner.lock().read_exact(buf)
    }
}

impl Write for LockedDiskCursor {
    fn write(&mut self, buf: &[u8]) -> Result<usize, DiskCursorIoError> {
        self.inner.lock().write(buf)
    }

    fn flush(&mut self) -> Result<(), DiskCursorIoError> {
        self.inner.lock().flush()
    }
}

impl Seek for LockedDiskCursor {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, DiskCursorIoError> {
        self.inner.lock().seek(pos)
    }
}

pub fn test_fatfs() {
    let start_sector: usize = 0;
    let storage = LockedDiskCursor::new(start_sector);

    let options = FsOptions::new();
    let fs = FileSystem::new(storage, options).expect("open fs");
    let root_cursor = fs.root_dir();

    // ls root
    for entry in root_cursor.iter() {
        let entry = entry.expect("Entry");
        print!("{} ", entry.file_name());
    }
    println!();
    println!("-----------");
    // create a file

    let path = "rust_fat32.txt";
    let mut file = root_cursor.create_file(path).expect("file");
    println!("create_file OK");
    file.write(b"fat32 write test").expect("file write");
    drop(file);

    // ls root
    for entry in root_cursor.iter() {
        let entry = entry.expect("Entry");
        print!("{} ", entry.file_name());
    }
    println!();
    println!("-----------");

    let mut file = root_cursor.open_file(path).expect("file");
    let mut buf: [u8; 16] = [0; 16];
    file.read(&mut buf).expect("file read");

    let content = String::from_utf8_lossy(&buf);
    println!("CONTENT: {:?}", content);
}

type InnerFatfs = FileSystem<DiskCursor, DefaultTimeProvider, LossyOemCpConverter>;

const MAX_FILE: usize = 128;

struct Fatfs<'a> {
    fs: InnerFatfs,
    fd_list: [bool; MAX_FILE],
    fd2file: BTreeMap<usize, File<'a, DiskCursor, DefaultTimeProvider, LossyOemCpConverter>>,
}

impl<'a> Fatfs<'a> {
    pub fn singleton() -> Arc<RwLock<Self>> {
        let fs = FileSystem::new(DiskCursor::new(0), FsOptions::new()).expect("FATFS init");
        let fd_list = [false; MAX_FILE];
        let fd2file = BTreeMap::new();
        Arc::new(RwLock::new(Fatfs {
            fs,
            fd_list,
            fd2file,
        }))
    }

    fn get_free_fd(&self) -> usize {
        for i in 0..self.fd_list.len() {
            if self.fd_list[i] == false {
                return i;
            }
        }
        panic!("Too Many Files");
    }
}

#[allow(dead_code)]
impl<'a> Fatfs<'a> {
    fn open<'s: 'a>(
        &'s mut self,
        path: &str,
        perms: FilePerms,
    ) -> Result<Box<dyn PosixFile>, FileError> {
        let fs = &self.fs;
        let root = fs.root_dir();
        let res = root.open_file(path);
        let fd = self.get_free_fd();
        match res {
            Ok(file) => {
                self.fd2file.insert(fd, file);
                let my_file = FatfsFile::new(fd, path);
                Ok(Box::new(my_file))
            }
            _ => {
                if !perms.creat {
                    Err(FileError::ENOENT)
                } else {
                    match root.create_file(path) {
                        Ok(file) => {
                            self.fd2file.insert(fd, file);
                            let my_file = FatfsFile::new(fd, path);
                            Ok(Box::new(my_file))
                        }
                        _ => Err(FileError::EOTHERS),
                    }
                }
            }
        }
    }

    fn unlink(&mut self, path: &str) -> Result<(), FileError> {
        let fs = &self.fs;
        let root = fs.root_dir();
        match root.remove(path) {
            Ok(_) => Ok(()),
            Err(_) => Err(FileError::ENOENT),
        }
    }
}

#[allow(dead_code)]
impl<'a> PosixFileSystemInner for Fatfs<'a> {
    fn close(&mut self, fd: usize) -> Result<(), FileError> {
        match self.fd2file.remove(&fd) {
            Some(file) => {
                drop(file);
                Ok(())
            }
            None => Err(FileError::ENOENT),
        }
    }

    fn read(&mut self, fd: usize, len: u32) -> Result<Vec<u8>, FileError> {
        match self.fd2file.get_mut(&fd) {
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

    fn write(&mut self, fd: usize, buf: &[u8]) -> Result<u64, FileError> {
        match self.fd2file.get_mut(&fd) {
            Some(file) => match file.write(buf) {
                Ok(res) => Ok(res as u64),
                Err(_) => Err(FileError::EOTHERS),
            },
            None => Err(FileError::ENOENT),
        }
    }

    fn lseek(&mut self, fd: usize, offset: isize, whence: SeekWhence) -> Result<usize, FileError> {
        match self.fd2file.get_mut(&fd) {
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
struct FatfsFile<'a> {
    fd: usize,
    name: String,
    handler: Arc<RwLock<Fatfs<'a>>>,
}

impl<'a> FatfsFile<'a> {
    pub fn new(fd: usize, name: &str) -> Self {
        FatfsFile {
            fd,
            name: name.to_string(),
            handler: Fatfs::singleton(),
        }
    }
}

impl<'a> PosixFile for FatfsFile<'a> {
    fn close(&mut self) -> Result<(), FileError> {
        self.handler.write().close(self.fd)
    }

    fn read(&mut self, len: u32) -> Result<Vec<u8>, FileError> {
        self.handler.write().read(self.fd, len)
    }

    fn write(&mut self, buf: &[u8]) -> Result<u64, FileError> {
        self.handler.write().write(self.fd, buf)
    }

    fn lseek(&mut self, offset: isize, whence: SeekWhence) -> Result<usize, FileError> {
        self.handler.write().lseek(self.fd, offset, whence)
    }
}
