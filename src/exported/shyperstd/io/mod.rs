use core::fmt;
use alloc::{string::String, vec::Vec};

mod bufreader;
mod prelude;
mod impls;

pub use bufreader::BufReader;

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

/// The `Read` trait allows for reading bytes from a source.
pub trait Read {
    /// Pull some bytes from this source into the specified buffer, returning
    /// how many bytes were read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Read all bytes until EOF in this source, placing them into `buf`.
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let start_len = buf.len();
        let mut probe = [0u8; 32];
        loop {
            match self.read(&mut probe) {
                Ok(0) => return Ok(buf.len() - start_len),
                Ok(n) => buf.extend_from_slice(&probe[..n]),
                Err(e) => return Err(e),
            }
        }
    }

    /// Read all bytes until EOF in this source, appending them to `buf`.
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        unsafe { append_to_string(buf, |b| self.read_to_end(b)) }
    }

    /// Read the exact number of bytes required to fill `buf`.
    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err("failed to fill whole buffer")
        } else {
            Ok(())
        }
    }
}

/// A trait for objects which are byte-oriented sinks.
pub trait Write {
    /// Write a buffer into this writer, returning how many bytes were written.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Flush this output stream, ensuring that all intermediately buffered
    /// contents reach their destination.
    fn flush(&mut self) -> Result;

    /// Attempts to write an entire buffer into this writer.
    fn write_all(&mut self, mut buf: &[u8]) -> Result {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err("failed to write whole buffer"),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Writes a formatted string into this writer, returning any error
    /// encountered.
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<()> {
        // Create a shim which translates a Write to a fmt::Write and saves
        // off I/O errors. instead of discarding them
        struct Adapter<'a, T: ?Sized + 'a> {
            inner: &'a mut T,
            error: Result<()>,
        }

        impl<T: Write + ?Sized> fmt::Write for Adapter<'_, T> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut output = Adapter {
            inner: self,
            error: Ok(()),
        };
        match fmt::write(&mut output, fmt) {
            Ok(()) => Ok(()),
            Err(..) => {
                // check if the error came from the underlying `Write` or not
                if output.error.is_err() {
                    output.error
                } else {
                    Err("formatter error")
                }
            }
        }
    }
}

/// The `Seek` trait provides a cursor which can be moved within a stream of
/// bytes.
pub trait Seek {
    /// Seek to an offset, in bytes, in a stream.
    ///
    /// A seek beyond the end of a stream is allowed, but behavior is defined
    /// by the implementation.
    ///
    /// If the seek operation completed successfully,
    /// this method returns the new position from the start of the stream.
    /// That position can be used later with [`SeekFrom::Start`].
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;

    /// Rewind to the beginning of a stream.
    ///
    /// This is a convenience method, equivalent to `seek(SeekFrom::Start(0))`.
    fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    /// Returns the current seek position from the start of the stream.
    ///
    /// This is equivalent to `self.seek(SeekFrom::Current(0))`.
    fn stream_position(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the [`Seek`] trait.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(u64),

    /// Sets the offset to the size of this object plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    End(i64),

    /// Sets the offset to the current position plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    Current(i64),
}

/// A `BufRead` is a type of `Read`er which has an internal buffer, allowing it
/// to perform extra ways of reading.
pub trait BufRead: Read {
    /// Returns the contents of the internal buffer, filling it with more data
    /// from the inner reader if it is empty.
    fn fill_buf(&mut self) -> Result<&[u8]>;

    /// Tells this buffer that `amt` bytes have been consumed from the buffer,
    /// so they should no longer be returned in calls to `read`.
    fn consume(&mut self, amt: usize);

    /// Check if the underlying `Read` has any data left to be read.
    fn has_data_left(&mut self) -> Result<bool> {
        self.fill_buf().map(|b| !b.is_empty())
    }

    /// Read all bytes into `buf` until the delimiter `byte` or EOF is reached.   
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        let mut read = 0;
        loop {
            let (done, used) = {
                let available = match self.fill_buf() {
                    Ok(n) => n,
                    // Err(Error::WouldBlock) => continue,
                    Err(e) => return Err(e),
                };
                match available.iter().position(|&b| b == byte) {
                    Some(i) => {
                        buf.extend_from_slice(&available[..=i]);
                        (true, i + 1)
                    }
                    None => {
                        buf.extend_from_slice(available);
                        (false, available.len())
                    }
                }
            };
            self.consume(used);
            read += used;
            if done || used == 0 {
                return Ok(read);
            }
        }
    }

    /// Read all bytes until a newline (the `0xA` byte) is reached, and append
    /// them to the provided `String` buffer.
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        unsafe { append_to_string(buf, |b| self.read_until(b'\n', b)) }
    }
}

unsafe fn append_to_string<F>(buf: &mut String, f: F) -> Result<usize>
where
    F: FnOnce(&mut Vec<u8>) -> Result<usize>,
{
    let old_len = buf.len();
    let buf = unsafe { buf.as_mut_vec() };
    let ret = f(buf)?;
    if core::str::from_utf8(&buf[old_len..]).is_err() {
        Err("stream did not contain valid UTF-8")
    } else {
        Ok(ret)
    }
}

/// Struct for poll result.
#[derive(Debug, Default)]
pub struct PollState {
    /// Object can be read now.
    pub readable: bool,
    /// Object can be writen now.
    pub writable: bool,
}
