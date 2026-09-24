use feagi_basis_collections::generic_data::par_data::ParData;
use feagi_basis_quantization::prelude::*;

create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of a single neuron
    pub CorticalAreaNeuronPotential
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron relative to its parent cortical area
    pub CorticalAreaNeuronLocalIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of neurons (not voxels) within a cortical area
    pub CorticalAreaNeuronCount
);

//region Linear Collections

/// Membrane potentials of the neurons in one cortical area, in the backing store `Store`.
pub type CorticalAreaNeuronData<QI, Store> =
ParData<CorticalAreaNeuronLocalIndex<QI>, Store>;

#[cfg(feature = "alloc")]
pub type CorticalAreaNeuronDataVector<QI, QP> =
ParData<CorticalAreaNeuronLocalIndex<QI>, Vec<CorticalAreaNeuronPotential<QP>>>;

pub type CorticalAreaNeuronDataArray<QI, QP, const N: usize> =
ParData<CorticalAreaNeuronLocalIndex<QI>, [CorticalAreaNeuronPotential<QP>; N]>;

pub type CorticalAreaNeuronDataSlice<'a, QI, QP> =
ParData<CorticalAreaNeuronLocalIndex<QI>, &'a [CorticalAreaNeuronPotential<QP>]>;

pub type CorticalAreaNeuronDataSliceMut<'a, QI, QP> =
ParData<CorticalAreaNeuronLocalIndex<QI>, &'a mut [CorticalAreaNeuronPotential<QP>]>;

//endregion

//region Dimensional Collections



//endregion

