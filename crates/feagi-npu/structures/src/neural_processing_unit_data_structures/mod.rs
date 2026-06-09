//! Neural Processing Unit Data Structures

mod neural_processing_unit;
mod dimensional_cortical_area;
mod synapse_mappings;
pub mod neuron;
pub mod wrapped_indexing;
pub mod cortical_area;





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

////////////////

// restraints
// neuron model quantization controls fcl in quant, membrane potential quant, neuron model / nmq,
// // this means number models * number quantization each dimensional neuron types
// // Needs to have a global cortical index, local type cortical index, global neuron indexes, local type neuron index, local cortical area neuron index
// // cortical area also needs to hold has neuron fired bool, fire ledger?
// // all fcls, neuron ins, membrane potentials need to be able to execute in parallel (global index lookup?)
// // when filling fcls, we technically only need the per cortical area fcl to me mut, reading from fclc is nonmut (fclc may be shared by cortical model type)
// // // this means multiple threads will be writing in multiple quantizations separately

// we need some sort of tables for evenly spreading out work acrss cortical areas, synapse maps? (not condensed?)

// neuron firing consolidation -> fill bitmask, count activity for cortical area
// // above is done per cortical area, but separate table indicates neuron count and cortical index mapping

// dense neuron firing table (cannot hold data due to various quantization levels)
// // simply an int array of (neuron model type) and (neuron model typed index) and other flags (mp charge?) to make it 8 elements

// global synapse index table
// // element -> source neuron typed index, source cortical area typed index, destination neuron typed index, axon bundle typed index,   , type specifiers for source / destination neurons and fclc type and axon bundle type,




// CA to CA table will exist without any typing information globally (trying to type between 2 CAs and synapse type will be a pita)
// // will hold nonplastic / plastic / fire ledger? // memory?// all-2-one // one-2-all // all-2-all mappings
// // one to ones need neuron ranges, which are in the correct quantized group




// tables: global cortical area index ->
// // local cortical area index
// // total number neurons of said area
// // cumulative number of neurons leading to said area
// //

