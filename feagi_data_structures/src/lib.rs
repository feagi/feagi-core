mod templates;
mod error;
mod feagi_signal;
mod feagi_json;
pub mod common_macros;
pub mod genomic;
pub mod neurons;

pub use templates::*;
pub use error::FeagiDataError as FeagiDataError;
pub use feagi_signal::{FeagiSignal, FeagiSignalIndex};
pub use feagi_json::FeagiJSON;

