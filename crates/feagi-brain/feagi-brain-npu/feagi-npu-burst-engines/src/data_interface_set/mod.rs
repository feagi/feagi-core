
pub use data_interface_set::DataInterfaceChannelSet;

/// Channels using mpmc channels from the `Flume` crate
#[cfg(feature = "std")]
pub use implementations::flume::FlumeDataInterfaceSet;

/// Channels using the channels from the embedded `Embassy` crate
pub use implementations::embassy_channel::EmbassyDataInterfaceSet;

mod data_interface_set;
mod implementations;
