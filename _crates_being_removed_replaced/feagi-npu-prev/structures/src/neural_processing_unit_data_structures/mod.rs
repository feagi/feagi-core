//! Neural Processing Unit Data Structures

//mod dimensional_cortical_area;
mod synapse_mappings;

//mod cortical_area_header;

pub mod wrappers;
pub mod burst_engine;
// differentiators
// 0 is alive [1 bit]
// 1 neuron membrane quant [3 bit]
// 2 neuron model [3 bit]
// 3 neuron model quant [3 bit] -> includes 1 neuron membrane quant
// 4 cortical area structure type (dimensional?) [2 bit]

// 5 synapse model/type [5 bit]
// 6 synapse quantization [3 bit]



// independent tables
// burst count arr (0 is the current burst, beloiw is by index unique, a delay
// fclc primary / secondary mapping tables -> global with cortical flag per element (duplicates of this with delay stuff tbd)
// fcl data (used in different steps) -> under division by mp quant
// mp data (..)  -> under division by mp quant

// cortical configs (used in different steps)





// temp before neuron fire -> fclc filled (sepoerate table logic)
// condense to fcl
//### fcl injection ( other device areas, NOT sensor!)
// condense fcl to dense neuron pointers (mp typed neuron index, mp typed cortical index, cortical flag)
// // iterate over fcl length array to do this, with data about fcl ranges per fcl index (seperate table) -> generic target index + range + u8 flag table for processing big vectors into small ones? (no dont, too specific)
// phase, Neuron firing
// dense neuron pointers

















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

