use feagi_structures::feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;

/// Base trait for Cortical Area Layouts, which describes how the neurons of a cortical area are
/// laid out and any other specific cortical parameters for that layout
pub trait CorticalLayoutBase<FGQ>:
where
    FGQ: FeagiGlobalQuantization,
{
    // contains post_synaptic_potential_should_be_uniform,
    // is_postsynaptic_potential_drive_by_membrane_potential, number_active_neurons_this_burst
    
    // Does NOT contain the base post synaptic potential! We dont want to deal with CBQ quantization!

    // Only contain usable data if on the CPU

    // Number of neurons contained should be accessible

    // NOTE: Do NOT use the base directly, use the one of the derived types (and the device)
}

/// Describes a cortical area of voxels of XYZ dimensions
pub trait CorticalLayoutDimensional<FGQ>:
CorticalLayoutBase<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{
    // Dimensions of cortical area (4D) should be accessible
}

// TODO other types of cortical areas?