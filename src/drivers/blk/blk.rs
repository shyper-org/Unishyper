// Feature bits
pub const VIRTIO_BLK_F_SIZE_MAX: usize = 1;
pub const VIRTIO_BLK_F_SEG_MAX: usize = 2;
pub const VIRTIO_BLK_F_GEOMETRY: usize = 4;
pub const VIRTIO_BLK_F_RO: usize = 5;
pub const VIRTIO_BLK_F_BLK_SIZE: usize = 6;
pub const VIRTIO_BLK_F_TOPOLOGY: usize = 10;
pub const VIRTIO_BLK_F_MQ: usize = 12;

// Legacy feature bits
pub const VIRTIO_BLK_F_BARRIER: usize = 0;
pub const VIRTIO_BLK_F_SCSI: usize = 7;
pub const VIRTIO_BLK_F_FLUSH: usize = 9;
pub const VIRTIO_BLK_F_CONFIG_WCE: usize = 11;

/* And this is the final byte of the write scatter-gather list */
pub const VIRTIO_BLK_S_OK: u8 = 0;
pub const VIRTIO_BLK_S_IOERR: u8 = 1;
pub const VIRTIO_BLK_S_UNSUPP: u8 = 2;
