//! Neural Processing Unit Data Structures

mod neural_processing_unit;
mod dimensional_cortical_area;
mod synapse_mappings;
// cortical areas stored in multiple arrays, chaining the fcls directly for next step

// fclc is global
// fcl, neuron model, neuron potential, neuron firing, compact firing, cortical context is cortical area owned
// smap is global

// starting at neuron input data (fcl) per cortical area needing to read from fcl
// in groups of cortical area PackSize, check each input,,
// //  if zero stop . If not, take value, execute neuron dynamics, with neuron
// // models stored in the cortical area struct, continue PackSizeTimes and update neuron firing
// // bitpack before continuing, also writing to cortical area owned neuron potential

// inject force fire, inject potential override

// count number firing

// Data exchange between devices

// extract motor voxel data (fold first if needed).

// compact if less than 25%? TODO

// iterate chained cortical areas, for firing neurons follow indexes through o-o s-map (read only)
// // write new potential to fclc index as per smap

// Optional prelimary fclc fold in (in case of massive fclc (structure defining how seperate)

// fold in fclc into cortical area fcl (from cortical area, chained)

