mod error;

pub mod traits_and_enums;
pub mod implementations;

pub use error::FeagiNetworkError;
pub use traits_and_enums::client::client_shared::FeagiClientConnectionState;
pub use traits_and_enums::server::server_shared::FeagiServerBindState;