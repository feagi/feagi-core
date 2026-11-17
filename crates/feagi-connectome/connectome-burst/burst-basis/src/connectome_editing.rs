use connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use connectome_basis::visualization::NeuronVisualizationVoxelByteIndex;
use connectome_models::neuron_models::models::{NMNeurogenesisStorageOrGenerator};

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
                $($StepSpecificName([<ConnectomeEditStep $StepSpecificName>]<FIQ>),)+
            }


            pub enum ConnectomeEditStepResultantEnum<FIQ: FeagiIndexQuantization> {
                $($StepSpecificName([<ConnectomeEditResultant $StepSpecificName>]<FIQ>),)+
            }
            
            
            $( _impl_connectome_edit_step_requirements!($StepSpecificName); )+

        }

    }
}


/// Defines some sort of edit step for the connectome, and the result that should be returned
pub trait ConnectomeEditStep<FIQ: FeagiIndexQuantization>:
Sized + Into<ConnectomeEditStepEnum<FIQ>> {
    /// What would be returned if everything went fine
    type Resultant: ConnectomeEditStepResultant<FIQ>;
}

/// What an edit step returns 
pub trait ConnectomeEditStepResultant<FIQ: FeagiIndexQuantization>:
Sized + Into<ConnectomeEditStepResultantEnum<FIQ>> {}

/// A variant of this is implemented for every edit step on an editable burst engine
pub trait ConnectomeEditStepHandler<FIQ: FeagiIndexQuantization, Edit: ConnectomeEditStep<FIQ>> {
    /// Handle running the edit
    async fn handle_edit(&mut self, step: Edit) -> Result<Edit::Resultant, ConnectomeEditError<FIQ>>;
}


// Generate `ConnectomeEditStepEnum` and `ConnectomeEditResultantEnum`
create_step_and_result_enums!{
    Nothing,
    CorticalAreaAdd,
}

pub trait ConnectomeEditStepEnumHandler<FIQ: FeagiIndexQuantization>:
Sized
+ ConnectomeEditStepHandler<FIQ, ConnectomeEditStepNothing>
+ ConnectomeEditStepHandler<FIQ, ConnectomeEditStepCorticalAreaAdd<FIQ>>
{
    async fn handle_connectome_edit_step_enum(&mut self, step: ConnectomeEditStepEnum<FIQ>)
        -> Result<ConnectomeEditStepResultantEnum<FIQ>, ConnectomeEditError<FIQ>> {
        
        match step {
            ConnectomeEditStepEnum::Nothing(a) => {self.handle_edit(a)}
            ConnectomeEditStepEnum::CorticalAreaAdd(a) => {self.handle_edit(a)}
        }
    }
}

// TODO -> Edit genome-prop-inplace, Variance,  Delete, Resize



//region Steps

//region Nothing

pub struct ConnectomeEditStepNothing;

impl<FIQ: FeagiIndexQuantization> ConnectomeEditStep<FIQ> for ConnectomeEditStepNothing {
    type Resultant = ConnectomeEditStepResultantNothing;
}

impl<FIQ: FeagiIndexQuantization> Into<ConnectomeEditStepEnum<FIQ>> for ConnectomeEditStepNothing {
    fn into(self) -> ConnectomeEditStepEnum<FIQ> { ConnectomeEditStepEnum::Nothing(self) }
}

pub struct ConnectomeEditStepResultantNothing;

impl<FIQ: FeagiIndexQuantization> ConnectomeEditStepResultant<FIQ> for ConnectomeEditStepResultantNothing {}

impl<FIQ: FeagiIndexQuantization> Into<ConnectomeEditStepResultantEnum<FIQ>> for ConnectomeEditStepResultantNothing {
    fn into(self) -> ConnectomeEditStepResultantEnum<FIQ> { ConnectomeEditStepResultantEnum::Nothing(self) }
}

//endregion


//region Cortical Areas

//region Add

pub struct ConnectomeEditStepCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
    pub cortical_area_neurogenesis: NMNeurogenesisStorageOrGenerator<FIQ>
}

impl<FIQ: FeagiIndexQuantization> ConnectomeEditStep<FIQ> for ConnectomeEditStepCorticalAreaAdd<FIQ> {
    type Resultant = ConnectomeEditResultantCorticalAreaAdd<FIQ>;
}

pub struct ConnectomeEditResultantCorticalAreaAdd<FIQ: FeagiIndexQuantization>
{
    pub new_cortical_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub visualization_start_index: NeuronVisualizationVoxelByteIndex<FIQ::NeuronIndexQuant>
}

impl<FIQ: FeagiIndexQuantization> ConnectomeEditStepResultant<FIQ> for ConnectomeEditResultantCorticalAreaAdd<FIQ> {}

impl<FIQ: FeagiIndexQuantization> Into<ConnectomeEditStepResultantEnum<FIQ>> for ConnectomeEditResultantCorticalAreaAdd<FIQ> {
    fn into(self) -> ConnectomeEditStepResultantEnum<FIQ> { ConnectomeEditStepResultantEnum::CorticalAreaAdd(self) }
}

//endregion


//endregion

//endregion

pub struct ConnectomeEditsReturn<FIQ: FeagiIndexQuantization> {
    completed: Vec<ConnectomeEditResultantCorticalAreaAdd<FIQ>>,
    result: Result<(), ConnectomeEditError<FIQ>>
}

impl<FIQ: FeagiIndexQuantization> ConnectomeEditsReturn<FIQ> {
    pub fn new(completed: Vec<ConnectomeEditResultantCorticalAreaAdd<FIQ>>, result: Result<(), ConnectomeEditError<FIQ>>) -> Self {
        Self { completed, result }
    }

    pub fn result(&self) -> &Result<(), ConnectomeEditError<FIQ>> {
        &self.result
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectomeEditError<FIQ: FeagiIndexQuantization> {
    #[error("Unknown Failure. Connectome Corruption assumed")]
    UnknownFailure,
    #[error("The burst engine can not index the cortical area at its index quantization. Nothing was changed")]
    ExceedsIndexQuantization,
}

impl<FIQ: FeagiIndexQuantization> ConnectomeEditError<FIQ> {
    fn is_corrupted(&self) -> bool {
        match &self {
            ConnectomeEditError::UnknownFailure => {true}
            _ => {false}
        }
    }
}