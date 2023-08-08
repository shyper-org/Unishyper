/* Define system wide error type */
pub type Error = usize;
pub const ERROR_INVARG: usize = 1;
pub const ERROR_OOM: usize = 2;
pub const ERROR_MEM_NOT_MAP: usize = 3;
pub const ERROR_INTERNAL: usize = 4;
pub const ERROR_DENIED: usize = 5;
pub const ERROR_HOLD_ON: usize = 6;
pub const ERROR_OOR: usize = 7;
pub const ERROR_PANIC: usize = 8;

/// The error type used by Unishyper.
///
/// Similar to [`std::io::ErrorKind`].
///
/// [`std::io::ErrorKind`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html
#[repr(i32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShyperError {
    /// A socket address could not be bound because the address is already in use elsewhere.
    AddrInUse = 1,
    /// An entity already exists, often a file.
    AlreadyExists,
    /// Bad address.
    BadAddress,
    /// Bad internal state.
    BadState,
    /// The connection was refused by the remote server,
    ConnectionRefused,
    /// The connection was reset by the remote server.
    ConnectionReset,
    /// A non-empty directory was specified where an empty directory was expected.
    DirectoryNotEmpty,
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: ShyperError::InvalidInput
    InvalidData,
    /// Invalid parameter/argument.
    InvalidInput,
    /// Input/output error.
    Io,
    /// The filesystem object is, unexpectedly, a directory.
    IsADirectory,
    /// Not enough space/cannot allocate memory.
    NoMemory,
    /// A filesystem object is, unexpectedly, not a directory.
    NotADirectory,
    /// The network operation failed because it was not connected yet.
    NotConnected,
    /// The requested entity is not found.
    NotFound,
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    /// Device or resource is busy.
    ResourceBusy,
    /// The underlying storage (typically, a filesystem) is full.
    StorageFull,
    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    UnexpectedEof,
    /// This operation is unsupported or unimplemented.
    Unsupported,
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    WouldBlock,
    /// An error returned when an operation could not be completed because a
    /// call to `write()` returned [`Ok(0)`](Ok).
    WriteZero,
}

impl ShyperError {
    /// Returns the error description.
    pub fn as_str(&self) -> &'static str {
        use ShyperError::*;
        match *self {
            AddrInUse => "Address in use",
            BadAddress => "Bad address",
            BadState => "Bad internal state",
            AlreadyExists => "Entity already exists",
            ConnectionRefused => "Connection refused",
            ConnectionReset => "Connection reset",
            DirectoryNotEmpty => "Directory not empty",
            InvalidData => "Invalid data",
            InvalidInput => "Invalid input parameter",
            Io => "I/O error",
            IsADirectory => "Is a directory",
            NoMemory => "Out of memory",
            NotADirectory => "Not a directory",
            NotConnected => "Not connected",
            NotFound => "Entity not found",
            PermissionDenied => "Permission denied",
            ResourceBusy => "Resource busy",
            StorageFull => "No storage space",
            UnexpectedEof => "Unexpected end of file",
            Unsupported => "Operation not supported",
            WouldBlock => "Operation would block",
            WriteZero => "Write zero",
        }
    }

    /// Returns the error code value in `i32`.
    pub const fn code(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for ShyperError {
    type Error = i32;

    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > 0 && value <= core::mem::variant_count::<ShyperError>() as i32 {
            Ok(unsafe { core::mem::transmute(value) })
        } else {
            Err(value)
        }
    }
}

impl From<ShyperError> for &str {
    #[inline]
    fn from(value: ShyperError) -> &'static str{
        value.as_str()
    }
}

impl core::fmt::Display for ShyperError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}