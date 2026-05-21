use feagi_structures::base_feagi_types::quantizable_types::QuantizableUIntType;
use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{NeuronMembranePotential, VoxelCoordinate, VoxelIndexCount};
use crate::dynamics::neuron::shared::neurons::{NeuronDataRef, NeuronDataRefMut, NeuronModelParametersTrait};

//region Enumerated
pub struct EnumeratedSingleNeuronVoxel<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> {
    voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    dimensions: &'a Dimensions,
    potential: &'a NeuronMembranePotential<CANQ::NeuronValueQuant>,
    model_parameters: &'a NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> EnumeratedSingleNeuronVoxel<'a, CANQ, NMP> {
    pub fn get_voxel_index(&self) -> &VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.voxel_index
    }

    pub fn get_voxel_coordinate(&self) -> VoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant> {
        self.neuron_collection_ref.get_cortical_area_voxel_dimensions()
            .linear_index_to_standard_voxel_coordinate(self.voxel_index) // TODO move voxel transfer to quantize unit, add density aware function
    }

    pub fn neuron_ref(&self) -> NeuronDataRef<'a, CANQ, NMP> {
        NeuronDataRef::new(self.potential, self.model_parameters)
    }
}

pub struct EnumeratedSingleNeuronVoxelMut<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> {
    voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    dimensions: &'a Dimensions,
    potential: &'a mut NeuronMembranePotential<CANQ::NeuronValueQuant>,
    model_parameters: &'a mut NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> EnumeratedSingleNeuronVoxelMut<'a, CANQ, NMP> {
    pub fn get_voxel_index(&self) -> &VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.voxel_index
    }

    pub fn get_voxel_coordinate(&self) -> VoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant> {
        self.neuron_collection_ref.get_cortical_area_voxel_dimensions()
            .linear_index_to_standard_voxel_coordinate(self.voxel_index) // TODO move voxel transfer to quantize unit, add density aware function
    }

    pub fn neuron_ref(&self) -> NeuronDataRef<'a, CANQ, NMP> {
        NeuronDataRef::new(self.potential, self.model_parameters)
    }

    pub fn neuron_ref_mut(&self) -> NeuronDataRefMut<'a, CANQ, NMP> {
        NeuronDataRefMut::new(self.potential, self.model_parameters)
    }
}




pub struct EnumeratedMultiNeuronVoxel<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> {
    voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    dimensions: &'a Dimensions,
    potentials: &'a [NeuronMembranePotential<CANQ::NeuronValueQuant>],
    model_parameters: &'a [NMP],
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> EnumeratedMultiNeuronVoxel<'a, CANQ, NMP, NeuronSetIndexTrait> {
    pub fn get_voxel_index(&self) -> &VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.voxel_index
    }

    pub fn get_voxel_coordinate(&self) -> VoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant> {
        self.neuron_collection_ref.get_cortical_area_voxel_dimensions()
            .linear_index_to_standard_voxel_coordinate(self.voxel_index) // TODO move voxel transfer to quantize unit, add density aware function
    }

    pub fn neuron_ref(&self) -> NeuronModelSlice<'a, CANQ, NMP> {
        NeuronModelSlice {
            neuron_potentials: self.potentials,
            get_model_parameters: self.model_parameters,
        }
    }
}

pub struct EnumeratedMultiNeuronVoxelMut<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> {
    voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    dimensions: &'a Dimensions,
    potentials: &'a mut [NeuronMembranePotential<CANQ::NeuronValueQuant>],
    model_parameters: &'a mut [NMP],
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> EnumeratedMultiNeuronVoxelMut<'a, CANQ, NMP, NeuronSetIndexTrait> {
    pub fn get_voxel_index(&self) -> &VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.voxel_index
    }

    pub fn get_voxel_coordinate(&self) -> VoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant> {
        self.neuron_collection_ref.get_cortical_area_voxel_dimensions()
            .linear_index_to_standard_voxel_coordinate(self.voxel_index) // TODO move voxel transfer to quantize unit, add density aware function
    }

    pub fn neuron_ref(&self) -> NeuronModelSlice<'a, CANQ, NMP> {
        NeuronModelSlice {
            neuron_potentials: self.potentials,
            get_model_parameters: self.model_parameters,
        }
    }

    pub fn neuron_ref_mut(&mut self) -> NeuronModelMutSlice<'a, CANQ, NMP> {
        NeuronModelMutSlice {
            neuron_potentials: self.potentials,
            get_model_parameters: self.model_parameters,
        }
    }
}

//endregion