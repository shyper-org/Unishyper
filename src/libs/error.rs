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