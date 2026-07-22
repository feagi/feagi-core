//! A flag in this context is a property that is bitpacked within an u8. Normally a bool, but
//! potentially an enum as well (called a packed enum). Flags allow the storage of many variables
//! in a small memory footprint, and are grouped together based on common access

/// Per cortical area flags that are used during the neuron burst phase of a burst
pub struct CorticalNeuronFlags(u8);
