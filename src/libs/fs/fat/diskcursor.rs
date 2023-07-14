use spin::Mutex;
use fatfs::{IoBase, IoError, Read, Write, Seek, SeekFrom};

use crate::libs::fs::fat::io::BlockCache;
use crate::libs::fs::interface::BlkIO;

pub const BSIZE: usize = 512;

#[derive(Debug)]
pub enum DiskCursorIoError {
    UnexpectedEof,
    WriteZero,
}

pub struct DiskCursor {
    pub sector: usize,
    pub offset: usize,
    // Block Cache
    pub cache: BlockCache,
}

impl DiskCursor {
    pub fn new(start_sector: usize) -> Self {
        DiskCursor {
            sector: start_sector,
            offset: 0,
            cache: BlockCache::new(),
        }
    }

    pub fn get_position(&self) -> usize {
        self.sector * BSIZE + self.offset
    }

    pub fn set_position(&mut self, position: usize) {
        self.sector = position / BSIZE;
        self.offset = position % BSIZE;
    }

    pub fn move_cursor(&mut self, amount: usize) {
        self.set_position(self.get_position() + amount)
    }
}

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
            let count = 1;
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
        if n != buf.len() {
            return Err(DiskCursorIoError::UnexpectedEof);
        }
        Ok(())
    }
}

impl Write for DiskCursor {
    fn write(&mut self, buf: &[u8]) -> Result<usize, DiskCursorIoError> {
        let mut i = 0;
        while i < buf.len() {
            let count = 1;
            let block = self.cache.get(self.sector, count);

            let data = block.get_data_mut(self.offset);
            if data.len() == 0 {
                break;
            }

            let end = (i + data.len()).min(buf.len());
            let len = end - i;
            data[..len].copy_from_slice(&buf[i..end]);

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

#[allow(unused)]
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
