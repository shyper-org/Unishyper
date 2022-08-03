use core::cmp::PartialEq;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

/// Virtio's feature bits inside an enum.
/// See Virtio specification v1.1. - 6
#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u64)]
pub enum Features {
    VIRTIO_F_RING_INDIRECT_DESC = 1 << 28,
    VIRTIO_F_RING_EVENT_IDX = 1 << 29,
    VIRTIO_F_VERSION_1 = 1 << 32,
    VIRTIO_F_ACCESS_PLATFORM = 1 << 33,
    VIRTIO_F_RING_PACKED = 1 << 34,
    VIRTIO_F_IN_ORDER = 1 << 35,
    VIRTIO_F_ORDER_PLATFORM = 1 << 36,
    VIRTIO_F_SR_IOV = 1 << 37,
    VIRTIO_F_NOTIFICATION_DATA = 1 << 38,
}

impl PartialEq<Features> for u64 {
    fn eq(&self, other: &Features) -> bool {
        self == other
    }
}

impl PartialEq<u64> for Features {
    fn eq(&self, other: &u64) -> bool {
        self == other
    }
}

impl From<Features> for u64 {
    fn from(val: Features) -> Self {
        match val {
            Features::VIRTIO_F_RING_INDIRECT_DESC => 1 << 28,
            Features::VIRTIO_F_RING_EVENT_IDX => 1 << 29,
            Features::VIRTIO_F_VERSION_1 => 1 << 32,
            Features::VIRTIO_F_ACCESS_PLATFORM => 1 << 33,
            Features::VIRTIO_F_RING_PACKED => 1 << 34,
            Features::VIRTIO_F_IN_ORDER => 1 << 35,
            Features::VIRTIO_F_ORDER_PLATFORM => 1 << 36,
            Features::VIRTIO_F_SR_IOV => 1 << 37,
            Features::VIRTIO_F_NOTIFICATION_DATA => 1 << 38,
        }
    }
}

impl BitOr for Features {
    type Output = u64;

    fn bitor(self, rhs: Self) -> Self::Output {
        u64::from(self) | u64::from(rhs)
    }
}

impl BitOr<Features> for u64 {
    type Output = u64;

    fn bitor(self, rhs: Features) -> Self::Output {
        self | u64::from(rhs)
    }
}

impl BitOrAssign<Features> for u64 {
    fn bitor_assign(&mut self, rhs: Features) {
        *self |= u64::from(rhs);
    }
}

impl BitAnd for Features {
    type Output = u64;

    fn bitand(self, rhs: Features) -> Self::Output {
        u64::from(self) & u64::from(rhs)
    }
}

impl BitAnd<Features> for u64 {
    type Output = u64;

    fn bitand(self, rhs: Features) -> Self::Output {
        self & u64::from(rhs)
    }
}

impl BitAndAssign<Features> for u64 {
    fn bitand_assign(&mut self, rhs: Features) {
        *self &= u64::from(rhs);
    }
}
