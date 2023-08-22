#[cfg(not(feature = "mpk"))]
mod dummy;

#[cfg(feature = "mpk")]
mod pkey;


#[cfg(not(feature = "mpk"))]
pub use dummy::*;

#[cfg(feature = "mpk")]
pub use pkey::*;
