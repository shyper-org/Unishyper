/* This marks a buffer as continuing via the next field */
pub const VRING_DESC_F_NEXT: u16 = 1;
/* This marks a buffer as write-only (otherwise read-only) */
pub const VRING_DESC_F_WRITE: u16 = 2;
/* This means the buffer contains a list of buffer descriptors */
pub const VRING_DESC_F_INDIRECT: u16 = 4;

pub const VIRTIO_RING_F_INDIRECT_DESC: usize = 28;
pub const VIRTIO_RING_F_EVENT_IDX: usize = 29;
