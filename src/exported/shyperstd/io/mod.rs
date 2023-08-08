
pub type Result<T = ()> = core::result::Result<T, &'static str>;

pub fn cvt(result: i32) -> Result<usize> {
    if result < 0 {
        Err("Shyper IO Error")
    } else {
        Ok(result as usize)
    }
}

#[cfg(feature = "fs")]
use ioslice::{IoSliceMut, IoSlice};

#[cfg(feature = "fs")]
pub(crate) fn default_read_vectored<F>(read: F, bufs: &mut [IoSliceMut<'_>]) -> Result<usize>
where
    F: FnOnce(&mut [u8]) -> Result<usize>,
{
    let buf = bufs
        .iter_mut()
        .find(|b| !b.is_empty())
        .map_or(&mut [][..], |b| &mut **b);
    read(buf)
}

#[cfg(feature = "fs")]
pub(crate) fn default_write_vectored<F>(write: F, bufs: &[IoSlice<'_>]) -> Result<usize>
where
    F: FnOnce(&[u8]) -> Result<usize>,
{
    let buf = bufs
        .iter()
        .find(|b| !b.is_empty())
        .map_or(&[][..], |b| &**b);
    write(buf)
}

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is based on the `std::io::SeekFrom` enum.
#[cfg(feature = "fs")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(u64),
    /// Sets the offset to the size of this object plus the specified number of bytes.
    End(i64),
    /// Sets the offset to the current position plus the specified number of bytes.
    Current(i64),
}