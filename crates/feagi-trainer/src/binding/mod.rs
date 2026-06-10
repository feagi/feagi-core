//! FEAGI binding layer — the runtime abstraction and the encoder/decoder/reward axes.
//!
//! This module is the seam between the Trainer and a live FEAGI runtime. It is deliberately
//! decoupled from any concrete neuron-data representation: [`runtime::FeagiRuntime`] exchanges
//! associated `SensoryFrame`/`MotorFrame` types, and the encoder/decoder plugins are generic
//! over those frames. That insulation lets the binding survive the in-flight NPU/quantization
//! refactor (which changes the concrete neuron-voxel types) without touching this contract.
//!
//! The encoder/decoder plugins are **selectors** over FEAGI's native coders (ADR-002,
//! Appendix B.1) — they configure and record a coder choice; they implement no spike-coding
//! math. The reward axis maps correctness/outcome onto FEAGI's native affect channels
//! (Pain/Pleasure/Fear/Hope) and is a first-class versioned axis (Appendix D.2).
//!
//! This slice (3b-1) provides the abstraction, the encoding-scheme registry (Appendix E),
//! and a concrete `RewardPolicy`. The concrete `ConnectorCache`-backed selectors and the
//! `FeagiRuntime` implementations (remote/embedded) land alongside the pinned connectome.

pub mod class_decoder;
pub mod decoder;
pub mod encoder;
pub mod encoding_scheme;
pub mod population_encoder;
pub mod profile;
#[cfg(feature = "remote-runtime")]
pub mod remote_runtime;
pub mod reward;
pub mod runtime;
pub mod stub_runtime;

pub use class_decoder::ClassDecoder;
pub use decoder::DecoderPlugin;
pub use encoder::EncoderPlugin;
pub use encoding_scheme::{BinSpacing, EncodingScheme, ResolvedEncodingScheme};
pub use population_encoder::PopulationEncoder;
pub use profile::{DecoderBindingProfile, EncoderBindingProfile};
#[cfg(feature = "remote-runtime")]
pub use remote_runtime::{RemoteFeagiRuntime, RemoteRuntimeConfig};
pub use reward::{AffectChannel, PainPleasureReward, RewardPolicy, RewardSignal};
pub use runtime::FeagiRuntime;
pub use stub_runtime::StubFeagiRuntime;
