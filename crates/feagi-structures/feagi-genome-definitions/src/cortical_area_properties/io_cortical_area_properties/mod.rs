

// NOTE: While we can do varying quantization, this stuff isnt really being thrown around in
// memory or transport in any large scale way, so the additional complexity just makes no sense.
// Furthermore, if different services use different quantizations, we risk problems

mod cortical_unit_index;
mod io_cortical_area_channels;
mod motor_cortical_unit;
mod sensory_cortical_unit;

pub mod io_cortical_area_configuration_flag;

