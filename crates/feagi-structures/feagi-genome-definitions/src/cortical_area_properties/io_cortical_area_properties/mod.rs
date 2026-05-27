

// NOTE: While we can do varying quantization, this stuff isnt really being thrown around in
// memory or transport in any large scale way, so the additional complexity just makes no sense.
// Furthermore, if different services use different quantizations, we risk problems

mod cortical_unit_index;
mod cortical_channel_index;

pub use cortical_unit_index::{CorticalUnitIndex, CorticalSubUnitIndex};
pub use cortical_channel_index::{CorticalChannelCount, CorticalChannelNeuronDepth};

