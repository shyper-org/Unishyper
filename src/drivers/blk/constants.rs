/* And this is the final byte of the write scatter-gather list */
#[allow(unused)]
pub const VIRTIO_BLK_S_OK: u8 = 0;
#[allow(unused)]
pub const VIRTIO_BLK_S_IOERR: u8 = 1;
#[allow(unused)]
pub const VIRTIO_BLK_S_UNSUPP: u8 = 2;

/// Status of a VirtIOBlk request.
#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(unused)]
pub enum RespStatus {
    /// Ok.
    Ok = 0,
    /// IoErr.
    IoErr = 1,
    /// Unsupported yet.
    Unsupported = 2,
    /// Not ready.
    _NotReady = 3,
}

pub const BLK_SIZE: usize = 512;

#[allow(dead_code, non_camel_case_types)]
#[repr(u32)]
#[derive(Debug)]
pub enum ReqType {
    VIRTIO_BLK_T_IN = 0,
    VIRTIO_BLK_T_OUT = 1,
    VIRTIO_BLK_T_FLUSH = 4,
    VIRTIO_BLK_T_DISCARD = 11,
    VIRTIO_BLK_T_WRITE_ZEROES = 13,
}

#[allow(dead_code, non_camel_case_types)]
pub enum Features {
    // Feature bits
    VIRTIO_BLK_F_SIZE_MAX = 1 << 1, // Maximum size of any single segment is in size_max
    VIRTIO_BLK_F_SEG_MAX = 1 << 2,  // Maximum number of segments in a request is in seg_max
    VIRTIO_BLK_F_GEOMETRY = 1 << 4, // Disk-style geometry specified in geometry
    VIRTIO_BLK_F_RO = 1 << 5,       // Device is read-only
    VIRTIO_BLK_F_BLK_SIZE = 1 << 6, // Block size of disk is in blk_size
    VIRTIO_BLK_F_FLUSH = 1 << 9,    // Cache flush command support
    VIRTIO_BLK_F_TOPOLOGY = 1 << 10, // Device exports information on optimal I/O alignment
    VIRTIO_BLK_F_CONFIG_WCE = 1 << 11, // Device exports information on optimal I/O alignment
    VIRTIO_BLK_F_MQ = 1 << 12, // Device can toggle its cache between writeback and writethrough modes
    VIRTIO_BLK_F_DISCARD = 1 << 13, // Device can support discard command, maximum discard sectors size in max_discard_sectors and maximum discard segment number in max_discard_seg.
    VIRTIO_BLK_F_WRITE_ZEROES = 1 << 14, //Device can support write zeroes command, maximum write zeroes VIRTIO_BLK_F_WRITE_ZEROES
    // Legacy feature bits
    VIRTIO_BLK_F_BARRIER = 1 << 0,
    VIRTIO_BLK_F_SCSI = 1 << 7,
    // In the legacy interface, VIRTIO_BLK_F_FLUSH was also called VIRTIO_BLK_F_WCE.
}

impl From<Features> for u64 {
    fn from(val: Features) -> Self {
        match val {
            Features::VIRTIO_BLK_F_SIZE_MAX => 1 << 1,
            Features::VIRTIO_BLK_F_SEG_MAX => 1 << 2,
            Features::VIRTIO_BLK_F_GEOMETRY => 1 << 4,
            Features::VIRTIO_BLK_F_RO => 1 << 5,
            Features::VIRTIO_BLK_F_BLK_SIZE => 1 << 6,
            Features::VIRTIO_BLK_F_FLUSH => 1 << 9,
            Features::VIRTIO_BLK_F_TOPOLOGY => 1 << 10,
            Features::VIRTIO_BLK_F_CONFIG_WCE => 1 << 11,
            Features::VIRTIO_BLK_F_MQ => 1 << 12,
            Features::VIRTIO_BLK_F_DISCARD => 1 << 13,
            Features::VIRTIO_BLK_F_WRITE_ZEROES => 1 << 14,
            Features::VIRTIO_BLK_F_BARRIER => 1 << 0,
            Features::VIRTIO_BLK_F_SCSI => 1 << 7,
        }
    }
}

impl core::fmt::Display for Features {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Features::VIRTIO_BLK_F_SIZE_MAX => write!(f, "VIRTIO_BLK_F_SIZE_MAX"),
            Features::VIRTIO_BLK_F_SEG_MAX => write!(f, "VIRTIO_BLK_F_SEG_MAX"),
            Features::VIRTIO_BLK_F_GEOMETRY => write!(f, "VIRTIO_BLK_F_GEOMETRY"),
            Features::VIRTIO_BLK_F_RO => write!(f, "VIRTIO_BLK_F_RO"),
            Features::VIRTIO_BLK_F_BLK_SIZE => write!(f, "VIRTIO_BLK_F_BLK_SIZE"),
            Features::VIRTIO_BLK_F_FLUSH => write!(f, "VIRTIO_BLK_F_FLUSH"),
            Features::VIRTIO_BLK_F_TOPOLOGY => write!(f, "VIRTIO_BLK_F_TOPOLOGY"),
            Features::VIRTIO_BLK_F_CONFIG_WCE => write!(f, "VIRTIO_BLK_F_CONFIG_WCE"),
            Features::VIRTIO_BLK_F_MQ => write!(f, "VIRTIO_BLK_F_MQ"),
            Features::VIRTIO_BLK_F_DISCARD => write!(f, "VIRTIO_BLK_F_DISCARD"),
            Features::VIRTIO_BLK_F_WRITE_ZEROES => write!(f, "VIRTIO_BLK_F_WRITE_ZEROES"),
            Features::VIRTIO_BLK_F_BARRIER => write!(f, "VIRTIO_BLK_F_BARRIER"),
            Features::VIRTIO_BLK_F_SCSI => write!(f, "VIRTIO_BLK_F_SCSI"),
        }
    }
}
