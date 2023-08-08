use core::fmt;
use core::hash::{Hash, Hasher};

use alloc::string::String;
use ioslice::{IoSlice, IoSliceMut};

use crate::libs::fs;
use crate::libs::fs::interface::{O_APPEND, O_CREAT, O_EXCL, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY};

use crate::exported::shyperstd::io::{self, cvt,SeekFrom};
use crate::exported::shyperstd::fd::FileDesc;

pub struct Path {
    inner: String,
}

pub fn cstr(path: &Path) -> io::Result<String> {
    use crate::libs::fs::FS_ROOT;
    use alloc::format;
    Ok(String::from(
        format!("{}{}", FS_ROOT, path.to_str().unwrap()).as_str(),
    ))
}

#[allow(unused)]
impl Path {
    fn as_u8_slice(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    // pub fn new<S: AsRef<String> + ?Sized>(s: &S) -> &Path {
    //     unsafe { &*(s.as_ref() as *const String as *const Path) }
    // }

    pub fn new(s: &str) -> Path {
        Self {
            inner: String::from(s),
        }
    }

    pub fn to_str(&self) -> Option<&str> {
        Some(self.inner.as_str())
    }
}

#[derive(Debug)]
pub struct File(FileDesc);

pub struct FileAttr(!);

pub struct ReadDir(!);

pub struct DirEntry(!);

#[derive(Clone, Debug)]
pub struct OpenOptions {
    // generic
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    // system-specific
    mode: i32,
}

pub struct FilePermissions(!);

pub struct FileType(!);

#[derive(Debug)]
pub struct DirBuilder {}

impl Clone for FileAttr {
    fn clone(&self) -> FileAttr {
        self.0
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        self.0
    }

    pub fn set_readonly(&mut self, _readonly: bool) {
        self.0
    }
}

impl Clone for FilePermissions {
    fn clone(&self) -> FilePermissions {
        self.0
    }
}

impl PartialEq for FilePermissions {
    fn eq(&self, _other: &FilePermissions) -> bool {
        self.0
    }
}

impl Eq for FilePermissions {}

impl fmt::Debug for FilePermissions {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.0
    }

    pub fn is_file(&self) -> bool {
        self.0
    }

    pub fn is_symlink(&self) -> bool {
        self.0
    }
}

impl Clone for FileType {
    fn clone(&self) -> FileType {
        self.0
    }
}

impl Copy for FileType {}

impl PartialEq for FileType {
    fn eq(&self, _other: &FileType) -> bool {
        self.0
    }
}

impl Eq for FileType {}

impl Hash for FileType {
    fn hash<H: Hasher>(&self, _h: &mut H) {
        self.0
    }
}

impl fmt::Debug for FileType {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        self.0
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            // generic
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            // system-specific
            mode: 0x777,
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    fn get_access_mode(&self) -> io::Result<i32> {
        match (self.read, self.write, self.append) {
            (true, false, false) => Ok(O_RDONLY),
            (false, true, false) => Ok(O_WRONLY),
            (true, true, false) => Ok(O_RDWR),
            (false, _, true) => Ok(O_WRONLY | O_APPEND),
            (true, _, true) => Ok(O_RDWR | O_APPEND),
            (false, false, false) => Err("invalid access mode"),
        }
    }

    fn get_creation_mode(&self) -> io::Result<i32> {
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return Err("invalid creation mode");
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return Err("invalid creation mode");
                }
            }
        }

        Ok(match (self.create, self.truncate, self.create_new) {
            (false, false, false) => 0,
            (true, false, false) => O_CREAT,
            (false, true, false) => O_TRUNC,
            (true, true, false) => O_CREAT | O_TRUNC,
            (_, _, true) => O_CREAT | O_EXCL,
        })
    }
}

impl File {
    pub fn open(path: &Path) -> io::Result<File> {
        let mut opts = OpenOptions::new();
        opts.read(true);
        let path = cstr(path)?;
        File::open_c(&path, &opts)
    }

    pub fn create(path: &Path) -> io::Result<File> {
        let mut opts = OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        let path = cstr(path)?;
        File::open_c(&path, &opts)
    }

    pub fn open_c(path: &str, opts: &OpenOptions) -> io::Result<File> {
        let mut flags = opts.get_access_mode()?;
        flags = flags | opts.get_creation_mode()?;

        let mode;
        if flags & O_CREAT == O_CREAT {
            mode = opts.mode;
        } else {
            mode = 0;
        }

        let fd = cvt(fs::open(path, flags, mode))?;
        Ok(File(FileDesc::new(fd as i32)))
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        unimplemented!()
    }

    pub fn fsync(&self) -> io::Result<()> {
        unimplemented!()
    }

    pub fn datasync(&self) -> io::Result<()> {
        self.fsync()
    }

    pub fn truncate(&self, _size: u64) -> io::Result<()> {
        unimplemented!()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        io::default_read_vectored(|buf| self.read(buf), bufs)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        false
    }

    // pub fn read_buf(&self, buf: &mut ReadBuf<'_>) -> io::Result<()> {
    // io::default_read_buf(|buf| self.read(buf), buf)
    //
    // https://doc.rust-lang.org/stable/std/io/struct.ReadBuf.html
    // }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        io::default_write_vectored(|buf| self.write(buf), bufs)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err("failed to write whole buffer");
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn seek(&self, _pos: SeekFrom) -> io::Result<u64> {
        unimplemented!()
    }

    pub fn duplicate(&self) -> io::Result<File> {
        unimplemented!()
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        unimplemented!()
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {}
    }

    pub fn mkdir(&self, _p: &Path) -> io::Result<()> {
        unimplemented!()
    }
}

pub fn readdir(_p: &Path) -> io::Result<ReadDir> {
    unimplemented!()
}

pub fn unlink(path: &Path) -> io::Result<()> {
    let name = cstr(path)?;
    let _ = cvt(fs::unlink(name.as_str()))?;
    Ok(())
}

pub fn rename(_old: &Path, _new: &Path) -> io::Result<()> {
    unimplemented!()
}

pub fn set_perm(_p: &Path, perm: FilePermissions) -> io::Result<()> {
    match perm.0 {}
}

pub fn rmdir(_p: &Path) -> io::Result<()> {
    unimplemented!()
}

pub fn remove_dir_all(_path: &Path) -> io::Result<()> {
    //unimplemented!()
    Ok(())
}

// pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
//     unimplemented!()
// }

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unimplemented!()
}

pub fn link(_original: &Path, _link: &Path) -> io::Result<()> {
    unimplemented!()
}

pub fn stat(_p: &Path) -> io::Result<FileAttr> {
    unimplemented!()
}

pub fn lstat(_p: &Path) -> io::Result<FileAttr> {
    unimplemented!()
}

// pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
//     unimplemented!()
// }
