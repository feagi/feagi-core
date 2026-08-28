/// Universal properties that all Cortical Mapping Entries have
#[derive(Clone, Copy, Debug)]
pub struct CorticalMappingEntryProperties {
    /// The delay in bursts from a firing being generated to arriving at the destination.
    /// 0 means none
    pub propagation_delay: u16,
    /// The firing of this synapse inhibits firing of the downstream neuron (generally a negative potential)
    pub is_inhibitory: bool,
}
