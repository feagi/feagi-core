//! All synapses need to support these configurations

pub struct CorticalMappingEntryConfiguration
{
    propagation_delay: u16,
    is_inhibitory: bool,
}