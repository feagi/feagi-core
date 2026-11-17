use connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use connectome_basis::visualization::NeuronVisualizationVoxelByteIndex;
use crate::neuron_models::models::{NMContainer, NMNeurogenesisStorageOrGenerator};

macro_rules! _impl_connectome_edit_step_requirements {
    {$StepSpecificName:ident} => {
        paste::paste! {
            
            impl<FIQ: FeagiIndexQuantization> ConnectomeEditStepType<FIQ> for [<ConnectomeEditStep $StepSpecificName>]<FIQ> {
                type ConnectomeEditResultant = [<ConnectomeEditResultant $StepSpecificName>]<FIQ>;
            }

            impl<FIQ: FeagiIndexQuantization> From<[<ConnectomeEditStep $StepSpecificName>]<FIQ>> for ConnectomeEditStepEnum<FIQ> {
                fn from(value: [<ConnectomeEditStep $StepSpecificName>]<FIQ>) -> Self {
                    ConnectomeEditStepEnum::$StepSpecificName(value)
                }
            }

            impl<FIQ: FeagiIndexQuantization> From<[<ConnectomeEditResultant $StepSpecificName>]<FIQ>> for ConnectomeEditResultantEnum<FIQ>
            {
                fn from(value: [<ConnectomeEditResultant $StepSpecificName>]<FIQ>) -> Self {
                    ConnectomeEditResultantEnum::$StepSpecificName(value)
                }
            }
        }
    };
}

/// Creates ConnectomeEditStepEnum<FIQ: FeagiIndexQuantization>
/// and ConnectomeEditResultantEnum<FIQ: FeagiIndexQuantization>
macro_rules! create_step_and_result_enums {
    {
        $($StepSpecificName:ident, )+
    } => {

        paste::paste! {

            pub enum ConnectomeEditStepEnum<FIQ: FeagiIndexQuantization> {
                /// Used by GenomeEditStep to skip over edits that dont affect the connectome
                Nothing,
                $($StepSpecificName([<ConnectomeEditStep $StepSpecificName>]<FIQ>),)+
            }


            pub enum ConnectomeEditResultantEnum<FIQ: FeagiIndexQuantization> {
                /// Used by GenomeEditStep to skip over edits that dont affect the connectome
                Nothing,
                $($StepSpecificName([<ConnectomeEditResultant $StepSpecificName>]<FIQ>),)+
            }
            
            
            $( _impl_connectome_edit_step_requirements!($StepSpecificName); )+

        }

    }
}


/// Defines some sort of edit step for the connectome, and the result that should be returned
pub trait ConnectomeEditStepType<FIQ: FeagiIndexQuantization>:
Sized + Into<ConnectomeEditStepEnum<FIQ>> {
    /// What would be returned if everything went fine
    type ConnectomeEditResultant: ConnectomeEditStepResultant<FIQ>;
}

/// What an edit step returns 
pub trait ConnectomeEditStepResultant<FIQ: FeagiIndexQuantization>:
Sized + Into<ConnectomeEditResultantEnum<FIQ>>
{}

// Generate `ConnectomeEditStepEnum` and `ConnectomeEditResultantEnum`
create_step_and_result_enums!{
    CorticalAreaAdd,
}

// TODO -> Edit genome-prop-inplace, Variance,  Delete, Resize



//region Cortical Areas

//region Add

pub struct ConnectomeEditStepCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
    pub cortical_area_neurogenesis: NMNeurogenesisStorageOrGenerator<FIQ>
}

pub struct ConnectomeEditResultantCorticalAreaAdd<FIQ: FeagiIndexQuantization>
{
    pub new_cortical_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub visualization_start_index: NeuronVisualizationVoxelByteIndex<FIQ::NeuronIndexQuant>
}

impl<FIQ: FeagiIndexQuantization> ConnectomeEditStepResultant<FIQ> for ConnectomeEditResultantCorticalAreaAdd<FIQ> {}

//endregion


//endregion


pub struct ConnectomeEditsReturn<FIQ: FeagiIndexQuantization> {
    completed: Vec<ConnectomeEditResultantCorticalAreaAdd<FIQ>>,
    result: Result<(), ConnectomeEditError>
}

impl<FIQ: FeagiIndexQuantization> ConnectomeEditsReturn<FIQ> {
    pub fn new(completed: Vec<ConnectomeEditResultantCorticalAreaAdd<FIQ>>, result: Result<(), ConnectomeEditError>) -> Self {
        Self { completed, result }
    }

    pub fn result(&self) -> &Result<(), ConnectomeEditError> {
        &self.result
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectomeEditError {
    #[error("Unknown Failure. Connectome Corruption assumed")]
    UnknownFailure,
    #[error("The burst engine can not index the cortical area at its index quantization. Nothing was changed")]
    ExceedsIndexQuantization,
}

impl ConnectomeEditError {
    fn is_corrupted(&self) -> bool {
        match &self {
            ConnectomeEditError::UnknownFailure => {true}
            _ => {false}
        }
    }
}