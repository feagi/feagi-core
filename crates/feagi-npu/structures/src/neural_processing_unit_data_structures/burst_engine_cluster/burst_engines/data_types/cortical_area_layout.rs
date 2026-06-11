
//region Cortical Area Layout Type

use core::marker::PhantomData;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::cpu_wrappers::NPUWrappedCorticalAreaDimensions;
use crate::neural_processing_unit_data_structures::packed_cortical_descriptor::PackedCorticalDescriptor;

/// Represents what type of cortical layout is being used in a cortical area, within 3 bits
/// (limiting to only 8 options)
#[repr(u8)]
#[derive(Copy, Clone, Default)]
pub enum CorticalAreaLayoutType {
    #[default]
    Dimensional = 0,
    Memory = 1,
    // 6 more possible
}

// NOTE: Can be cast directly as u8 with the right bitfield for NeuronModelCorticalDescriptors!

impl From<&PackedCorticalDescriptor> for CorticalAreaLayoutType {
    fn from(packed_cortical_descriptor: &PackedCorticalDescriptor) -> Self
    {
        const BYTE_MASK: u8 = 7; // Last 3 bits
        let bits: u8 = packed_cortical_descriptor.into();
        match bits {
            0 => CorticalAreaLayoutType::Dimensional, // 0 0 0
            1 => CorticalAreaLayoutType::Memory, // 0 0 1
            _ => panic!("Nonvalid value decoded for CorticalAreaLayoutType or not yet implemented!"),
        }

    }
}

//endregion

/// Base trait for Cortical Area Layouts, which describes how the neurons of a cortical area are
/// laid out and any other specific cortical parameters for that layout
pub trait CorticalLayoutBase<FGQ, CPQ>:
where
    FGQ: FeagiGlobalQuantization,
    CPQ: CorticalPotentialQuantization
{
    // contains the post_synaptic_potential_base, post_synaptic_potential_should_be_uniform,
    // is_postsynaptic_potential_drive_by_membrane_potential, number_active_neurons_this_burst

    // Only contain usable data if on the CPU

    // Number of neurons contained should be accessible

    // NOTE: Do NOT use the base directly, use the one of the derived types (and the device)
}

/// Describes a cortical area of voxels of XYZ dimensions
pub trait CorticalLayoutDimensional<FGQ, CPQ>:
CorticalLayoutBase<FGQ, CPQ>
where
    FGQ: FeagiGlobalQuantization,
    CPQ: CorticalPotentialQuantization
{
    // Dimensions of cortical area (4D) should be accessible
}

// TODO other types of cortical areas?

//region CPU Implementations

#[repr(C)]
pub struct CorticalLayoutDimensionalCPU<FGQ, CPQ>
where
    FGQ: FeagiGlobalQuantization,
    CPQ: CorticalPotentialQuantization
{
    pub dimensions: NPUWrappedCorticalAreaDimensions<FGQ::NeuronIndexCountQuant>,
    _p: PhantomData<CPQ>,
}

impl<FGQ, CPQ> CorticalLayoutBase<FGQ, CPQ> for CorticalLayoutDimensionalCPU<FGQ, CPQ>
where FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization, {}

impl<FGQ, CPQ> CorticalLayoutDimensional<FGQ, CPQ> for CorticalLayoutDimensionalCPU<FGQ, CPQ>
where FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization, {}


//endregion