use crate::generic_collections::generic_par_data::linear::{ParDataArray, ParDataSlice, ParDataSliceMut, ParDataVector};
use crate::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
use crate::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use crate::values::quantizable::QuantizedUnsignedIntegerTrait;



pub type LinearCorticalNeuronPotentialVector<QI: QuantizedUnsignedIntegerTrait, Q: MembranePotentialQuantization> =
    ParDataVector<CorticalNeuronLocalIndex<QI>, CorticalNeuronPotential<Q::MembranePotentialQuant>>;

pub type LinearCorticalNeuronPotentialSlice<'a, QI: QuantizedUnsignedIntegerTrait, Q: MembranePotentialQuantization> =
    ParDataSlice<'a, CorticalNeuronLocalIndex<QI>, CorticalNeuronPotential<Q::MembranePotentialQuant>>;

pub type LinearCorticalNeuronPotentialSliceMut<'a, QI: QuantizedUnsignedIntegerTrait, Q: MembranePotentialQuantization> =
    ParDataSliceMut<'a, CorticalNeuronLocalIndex<QI>, CorticalNeuronPotential<Q::MembranePotentialQuant>>;

pub type LinearCorticalNeuronPotentialArray<QI: QuantizedUnsignedIntegerTrait, Q: MembranePotentialQuantization, const N: usize> =
    ParDataArray<CorticalNeuronLocalIndex<QI>, CorticalNeuronPotential<Q::MembranePotentialQuant>, N>;
