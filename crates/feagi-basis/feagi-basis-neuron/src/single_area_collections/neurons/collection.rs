use feagi_basis_collections::generic_data::par_data::ParData;

use crate::single_area_collections::neurons::context::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};

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
