/* And this is the final byte of the write scatter-gather list */
pub const VIRTIO_BLK_S_OK: u8 = 0;
pub const VIRTIO_BLK_S_IOERR: u8 = 1;
pub const VIRTIO_BLK_S_UNSUPP: u8 = 2;

#[allow(dead_code, non_camel_case_types)]
pub enum Features {
    // Feature bits
    VIRTIO_BLK_F_SIZE_MAX = 1 << 1,
    VIRTIO_BLK_F_SEG_MAX = 1 << 2,
    VIRTIO_BLK_F_GEOMETRY = 1 << 4,
    VIRTIO_BLK_F_RO = 1 << 5,
    VIRTIO_BLK_F_BLK_SIZE = 1 << 6,
    VIRTIO_BLK_F_FLUSH = 1 << 9,
    VIRTIO_BLK_F_TOPOLOGY = 1 << 10,
    VIRTIO_BLK_F_CONFIG_WCE = 1 << 11,
    VIRTIO_BLK_F_MQ = 1 << 12,
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
            Features::VIRTIO_BLK_F_BARRIER => write!(f, "VIRTIO_BLK_F_BARRIER"),
            Features::VIRTIO_BLK_F_SCSI => write!(f, "VIRTIO_BLK_F_SCSI"),
        }
    }
}
