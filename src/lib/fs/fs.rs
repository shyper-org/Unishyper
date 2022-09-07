use alloc::{collections::BTreeMap, string::String, boxed::Box};
use core::{ops::Deref, cell::RefCell};

use crate::spinlock::Spinlock;

use super::interface::{PosixFileSystem, PosixFile, FilePerms, FileError};

pub static FILESYSTEM: Spinlock<Filesystem> = Spinlock::new(Filesystem::new());

pub struct Filesystem {
    // Keep track of mount-points
    mounts: BTreeMap<String, Box<dyn PosixFileSystem + Send>>,

    // Keep track of open files
    files: RefCell<BTreeMap<u64, Box<dyn PosixFile + Send>>>,
}

impl Filesystem {
    pub const fn new() -> Self {
        Self {
            mounts: BTreeMap::new(),
            files: RefCell::new(BTreeMap::new()),
        }
    }

    /// Returns next free file-descriptor. We map index in files BTreeMap as fd's.
    /// Done determining the current biggest stored index.
    /// This is efficient, since BTreeMap's iter() calculates min and max key directly.
    /// see <https://github.com/rust-lang/rust/issues/62924>
    fn assign_new_fd(&self) -> u64 {
        // BTreeMap has efficient max/min index calculation. One way to access these is the following iter.
        // Add 1 to get next never-assigned fd num
        if let Some((fd, _)) = self.files.borrow().iter().next_back() {
            fd + 1
        } else {
            3 // start at 3, to reserve stdin/out/err
        }
    }

    /// parses path `/MOUNTPOINT/internal-path` into mount-filesystem and internal_path
    /// Returns (PosixFileSystem, internal_path) or Error on failure.
    fn parse_path<'b, 'a>(
        &'a self,
        path: &'b str,
    ) -> Result<(&'a (dyn PosixFileSystem + Send), &'b str), FileError> {
        let mut pathsplit = path.splitn(3, '/');

        if path.starts_with('/') {
            pathsplit.next(); // empty, since first char is /

            let mount = pathsplit.next().unwrap();
            let internal_path = pathsplit.next().unwrap();
            if let Some(fs) = self.mounts.get(mount) {
                return Ok((fs.deref(), internal_path));
            }

            warn!(
                "Trying to open file on non-existing mount point '{}'!",
                mount
            );
        } else {
            let mount = ".";
            let internal_path = pathsplit.next().unwrap();

            debug!(
                "Assume that the directory '{}' is used as mount point!",
                mount
            );

            if let Some(fs) = self.mounts.get(mount) {
                return Ok((fs.deref(), internal_path));
            }

            info!(
                "Trying to open file on non-existing mount point '{}'!",
                mount
            );
        }

        Err(FileError::ENOENT)
    }

    pub fn mount(
        &mut self,
        mntpath: &str,
        mntobj: Box<dyn PosixFileSystem + Send>,
    ) -> Result<(), ()> {
        use alloc::borrow::ToOwned;

        info!("Mounting {}", mntpath);
        if mntpath.contains('/') {
            warn!(
                "Trying to mount at '{}', but slashes in name are not supported!",
                mntpath
            );
            return Err(());
        }

        // if mounts contains path already abort
        if self.mounts.contains_key(mntpath) {
            warn!("Mountpoint already exists!");
            return Err(());
        }

        // insert filesystem into mounts, done
        self.mounts.insert(mntpath.to_owned(), mntobj);
        Ok(())
    }

    /// Tries to open file at given path (/MOUNTPOINT/internal-path).
    /// Looks up MOUNTPOINT in mounted dirs, passes internal-path to filesystem backend
    /// Returns the file descriptor of the newly opened file, or an error on failure
    pub fn open(&mut self, path: & str, perms: FilePerms) -> Result<u64, FileError> {
        debug!("Opening file {} {:?}", path, perms);
        let (fs, internal_path) = self.parse_path(path)?;
        let fd = self.assign_new_fd();
        let file = fs.open(internal_path, perms, fd as usize)?;
        self.files.borrow_mut().insert(fd, file);
        Ok(fd)
    }

    pub fn close(&mut self, fd: u64) {
        debug!("Closing fd {}", fd);
        if let Some(file) = self.files.borrow_mut().get_mut(&fd) {
            file.close().unwrap(); // TODO: handle error
        }
        self.files.borrow_mut().remove(&fd);
    }

    /// Unlinks a file given by path.
    pub fn unlink(&mut self, path: &str) -> Result<(), FileError> {
        info!("Unlinking file {}", path);
        let (fs, internal_path) = self.parse_path(path)?;
        fs.unlink(internal_path)?;
        Ok(())
    }

    /// Run closure on file referenced by file descriptor.
    pub fn fd_op(&mut self, fd: u64, f: impl FnOnce(&mut Box<dyn PosixFile + Send>)) {
        f(self.files.borrow_mut().get_mut(&fd).unwrap());
    }
}
