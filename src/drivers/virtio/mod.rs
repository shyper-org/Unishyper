use cfg_if::cfg_if;

pub mod env;
/// A module containing Virtio's feature bits.
pub mod features;

// #[cfg(any(feature = "net", feature = "fat"))]
//

#[cfg(feature = "mmio")]
pub mod mmio;

cfg_if! {
    if #[cfg(any(feature = "net", feature = "fat"))] {
        pub mod transport;
        pub mod virtqueue;
    }
}

pub fn init_drivers() {
    cfg_if! {
        if #[cfg(feature = "pci")] {
            crate::drivers::pci::init_drivers();
        } else {
            mmio::init_drivers();
        }
    }
}

pub const VIRTIO_MAX_QUEUE_SIZE: u16 = 1024;

pub mod error {
    #[cfg(feature = "pci")]
    use crate::drivers::pci::error::PciError;
    #[cfg(feature = "fat")]
    use crate::drivers::blk::virtio_blk::error::VirtioBlkError;
    #[cfg(feature = "net")]
    pub use crate::drivers::net::virtio_net::error::VirtioNetError;
    use core::fmt;

    #[derive(Debug)]
    pub enum VirtioError {
        #[cfg(feature = "pci")]
        FromPci(PciError),
        DevNotSupported(u16),
        #[cfg(feature = "net")]
        NetDriver(VirtioNetError),
        #[cfg(feature = "fat")]
        BlkDriver(VirtioBlkError),
        #[allow(unused)]
        Unknown,
    }

    impl fmt::Display for VirtioError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                VirtioError::Unknown => write!(f, "Driver failed to initialize virtio device due to unknown reasosn!"),
                #[cfg(feature = "pci")]
				VirtioError::FromPci(pci_error) => match pci_error {
                    PciError::General(id) => write!(f, "Driver failed to initialize device with id: {id:#x}. Due to unknown reasosn!"),
                    PciError::NoBar(id ) => write!(f, "Driver failed to initialize device with id: {id:#x}. Reason: No BAR's found."), 
                    PciError::NoCapPtr(id) => write!(f, "Driver failed to initialize device with id: {id:#x}. Reason: No Capabilities pointer found."),
                    PciError::BadCapPtr(id) => write!(f, "Driver failed to initialize device with id: {id:#x}. Reason: Malformed Capabilities pointer."),
                    PciError::NoVirtioCaps(id) => write!(f, "Driver failed to initialize device with id: {id:#x}. Reason: No Virtio capabilities were found."),
                },
                VirtioError::DevNotSupported(id) => write!(f, "Device with id {:#x} not supported.", id),
                #[cfg(feature = "net")]
                VirtioError::NetDriver(net_error) => match net_error {
                    VirtioNetError::General => write!(f, "Virtio network driver failed due to unknown reasons!"),
                    VirtioNetError::NoDevCfg(id) => write!(f, "Network driver failed, for device {:x}, due to a missing or malformed device config!", id),
                    VirtioNetError::NoComCfg(id) =>  write!(f, "Network driver failed, for device {:x}, due to a missing or malformed common config!", id),
                    VirtioNetError::NoIsrCfg(id) =>  write!(f, "Network driver failed, for device {:x}, due to a missing or malformed ISR status config!", id),
                    VirtioNetError::NoNotifCfg(id) =>  write!(f, "Network driver failed, for device {:x}, due to a missing or malformed notification config!", id),
                    VirtioNetError::FailFeatureNeg(id) => write!(f, "Network driver failed, for device {:x}, device did not acknowledge negotiated feature set!", id),
                    VirtioNetError::FeatReqNotMet(feats) => write!(f, "Network driver tried to set feature bit without setting dependency feature. Feat set: {:x}", u64::from(*feats)),
                    VirtioNetError::IncompFeatsSet(drv_feats, dev_feats) => write!(f, "Feature set: {:x} , is incompatible with the device features: {:x}", u64::from(*drv_feats), u64::from(*dev_feats)),
                    VirtioNetError::ProcessOngoing => write!(f, "Driver performed an unsuitable operation upon an ongoging transfer."),
					VirtioNetError::Unknown => write!(f, "Virtio network driver failed due unknown reason!"),
                },
                #[cfg(feature = "fat")]
				VirtioError::BlkDriver(blk_error) => match blk_error {
					VirtioBlkError::General => write!(f, "Virtio block driver failed due to unknown reasons!"),
					VirtioBlkError::NoDevCfg(id)=> write!(f, "Virtio block driver failed, for device {:x}, due to a missing or malformed device config!", id),
					VirtioBlkError::NoComCfg(id)=> write!(f, "Virtio block driver failed, for device {:x}, due to a missing or malformed common config!", id),
					VirtioBlkError::NoIsrCfg(id)=> write!(f, "Virtio block driver failed, for device {:x}, due to a missing or malformed ISR status config!", id),
					VirtioBlkError::NoNotifCfg(id)=> write!(f, "Virtio block driver failed, for device {:x}, due to a missing or malformed notification config!", id),
					VirtioBlkError::ProcessOngoing=> write!(f, "Driver performed an unsuitable operation upon an ongoging transfer."),
					VirtioBlkError::Unknown=> write!(f, "Virtio block driver failed due to unknown reasons!"),
				}
            }
        }
    }
}

/// A module containing virtios device specfific information.
pub mod device {
    /// An enum of the device's status field interpretations.
    #[allow(dead_code, non_camel_case_types, clippy::upper_case_acronyms)]
    #[derive(Clone, Copy, Debug)]
    #[repr(u8)]
    pub enum Status {
        ACKNOWLEDGE = 1,
        DRIVER = 2,
        DRIVER_OK = 4,
        FEATURES_OK = 8,
        DEVICE_NEEDS_RESET = 64,
        FAILED = 128,
    }

    impl From<Status> for u8 {
        fn from(stat: Status) -> Self {
            match stat {
                Status::ACKNOWLEDGE => 1,
                Status::DRIVER => 2,
                Status::DRIVER_OK => 4,
                Status::FEATURES_OK => 8,
                Status::DEVICE_NEEDS_RESET => 64,
                Status::FAILED => 128,
            }
        }
    }

    impl From<Status> for u32 {
        fn from(stat: Status) -> Self {
            match stat {
                Status::ACKNOWLEDGE => 1,
                Status::DRIVER => 2,
                Status::DRIVER_OK => 4,
                Status::FEATURES_OK => 8,
                Status::DEVICE_NEEDS_RESET => 64,
                Status::FAILED => 128,
            }
        }
    }

    /// Empty trait to unify all device specific configuration structs.
    pub trait DevCfg {}
}
