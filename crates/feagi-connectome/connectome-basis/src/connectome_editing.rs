use crate::engine::engine_change_command::EngineChangeCommandOtherError;
use crate::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use crate::quantization_levels::FeagiIndexQuantization;
use crate::visualization::NeuronVisualizationVoxelByteIndex;

macro_rules! _impl_connectome_edit_step_requirements {
    {$StepSpecificName:ident} => {
        paste::paste! {



            impl<FIQ: FeagiIndexQuantization, CNMT: ConnectomeNeuronModelTypes<FIQ>> ConnectomeEditStepType<FIQ, CNMT> for [<ConnectomeEditStep $StepSpecificName>]<FIQ, CNMT> {
                type ConnectomeEditResultant = [<ConnectomeEditResultant $StepSpecificName>]<FIQ>;
                type ConnectomeEditError = [<ConnectomeEditError $StepSpecificName>];
            }

            impl<FIQ: FeagiIndexQuantization, CNMT: ConnectomeNeuronModelTypes<FIQ>> From<[<ConnectomeEditStep $StepSpecificName>]<FIQ, CNMT>> for ConnectomeEditStepEnum<FIQ, CNMT> {
                fn from(value: [<ConnectomeEditStep $StepSpecificName>]<FIQ, CNMT>) -> Self {
                    ConnectomeEditStepEnum::$StepSpecificName(value)
                }
            }

            impl<FIQ: FeagiIndexQuantization> From<[<ConnectomeEditResultant $StepSpecificName>]<FIQ>> for ConnectomeEditResultantEnum<FIQ>
            {
                fn from(value: [<ConnectomeEditResultant $StepSpecificName>]<FIQ>) -> Self {
                    ConnectomeEditResultantEnum::$StepSpecificName(Ok(value))
                }
            }

            impl<FIQ: FeagiIndexQuantization> From<[<ConnectomeEditError $StepSpecificName>]> for ConnectomeEditResultantEnum<FIQ>
            {
                fn from(value: [<ConnectomeEditError $StepSpecificName>]) -> Self {
                    ConnectomeEditResultantEnum::$StepSpecificName(Err(value))
                }
            }



        }
    };
}

/// Creates ConnectomeEditStepEnum<FIQ: FeagiIndexQuantization, CNMT: ConnectomeNeuronModelTypes<FIQ>>
/// and ConnectomeEditResultantEnum<FIQ: FeagiIndexQuantization>
macro_rules! create_step_and_result_enums {
    {
        $($StepSpecificName:ident, )+
    } => {

        paste::paste! {

            pub enum ConnectomeEditStepEnum<FIQ: FeagiIndexQuantization, CNMT: ConnectomeNeuronModelTypes<FIQ>> {
                /// Used by GenomeEditStep to skip over edits that dont affect the connectome
                Nothing,
                $($StepSpecificName([<ConnectomeEditStep $StepSpecificName>]<FIQ, CNMT>),)+
            }


            pub enum ConnectomeEditResultantEnum<FIQ: FeagiIndexQuantization> {
                /// Used by GenomeEditStep to skip over edits that dont affect the connectome
                Nothing,
                $($StepSpecificName(Result<[<ConnectomeEditResultant $StepSpecificName>]<FIQ>, [<ConnectomeEditError $StepSpecificName>]>),)+
            }



            impl<FIQ: FeagiIndexQuantization> ConnectomeEditResultantEnum<FIQ> {
                /// True if the edit step failed. A failed step stops the rest of its batch
                pub fn is_err(&self) -> bool {
                    match self {
                        ConnectomeEditResultantEnum::Nothing => false,
                        $(ConnectomeEditResultantEnum::$StepSpecificName(result) => result.is_err(),)+
                    }
                }

                /// True if the edit step failed in a way that likely left the connectome corrupted
                pub fn is_connectome_corrupted(&self) -> bool {
                    match self {
                        ConnectomeEditResultantEnum::Nothing => false,
                        $(ConnectomeEditResultantEnum::$StepSpecificName(result) => {
                            result.as_ref().is_err_and(|error| error.is_connectome_corrupted())
                        })+
                    }
                }
            }

            $( _impl_connectome_edit_step_requirements!($StepSpecificName); )+

        }

    }
}


/// Since the Neuron Models are made in higher level crates, we need to use this to transfer those down
pub trait ConnectomeNeuronModelTypes<FIQ: FeagiIndexQuantization> {
    type CorticalAreaNeurogenesis: Send;
}

/// Defines some sort of edit step for the connectome, and the result that should be returned
pub trait ConnectomeEditStepType<FIQ: FeagiIndexQuantization, CNMT: ConnectomeNeuronModelTypes<FIQ>>:
Sized + Into<ConnectomeEditStepEnum<FIQ, CNMT>> {
    /// What would be returned if everything went fine
    type ConnectomeEditResultant: ConnectomeEditStepResultant<FIQ>;
    /// What is returned if something when wrong
    type ConnectomeEditError: ConnectomeEditError + Into<ConnectomeEditResultantEnum<FIQ>>;
}

pub trait ConnectomeEditStepResultant<FIQ: FeagiIndexQuantization>:
Sized + Into<ConnectomeEditResultantEnum<FIQ>>
{}


pub trait ConnectomeEditError: core::error::Error{
    /// Return true if the error likely caused connectome corruption
    fn is_connectome_corrupted(&self) -> bool;
}


// Generate `ConnectomeEditStepEnum` and `ConnectomeEditResultantEnum`
create_step_and_result_enums!{
    CorticalAreaAdd,
}

// TODO -> Edit genome-prop-inplace, Variance,  Delete, Resize



//region Cortical Areas

pub struct ConnectomeEditStepCorticalAreaAdd<FIQ: FeagiIndexQuantization, CNMT: ConnectomeNeuronModelTypes<FIQ>> {
    pub cortical_area_neurogenesis: CNMT::CorticalAreaNeurogenesis,
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization, CNMT: ConnectomeNeuronModelTypes<FIQ>> ConnectomeEditStepCorticalAreaAdd<FIQ, CNMT> {
    /// Adds a cortical area from its neurogenesis, at whichever neuron model it uses
    pub fn new(cortical_area_neurogenesis: CNMT::CorticalAreaNeurogenesis) -> Self {
        Self {
            cortical_area_neurogenesis,
            _p: core::marker::PhantomData,
        }
    }
}

pub struct ConnectomeEditResultantCorticalAreaAdd<FIQ: FeagiIndexQuantization>
{
    pub new_cortical_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub visualization_start_index: NeuronVisualizationVoxelByteIndex<FIQ::NeuronIndexQuant>
}

impl<FIQ: FeagiIndexQuantization> ConnectomeEditStepResultant<FIQ> for ConnectomeEditResultantCorticalAreaAdd<FIQ> {}

#[derive(Debug, thiserror::Error)]
pub enum ConnectomeEditErrorCorticalAreaAdd {
    #[error("Unknown Failure. Connectome Corruption assumed")]
    UnknownFailure,
    #[error("The burst engine can not index the cortical area at its index quantization. Nothing was changed")]
    ExceedsIndexQuantization,
}

impl ConnectomeEditError for ConnectomeEditErrorCorticalAreaAdd {
    fn is_connectome_corrupted(&self) -> bool {
        match &self {
            ConnectomeEditErrorCorticalAreaAdd::UnknownFailure => true,
            ConnectomeEditErrorCorticalAreaAdd::ExceedsIndexQuantization => false,
        }
    }
}



//endregion


/*
#[derive(Debug, thiserror::Error)]
pub enum ConnectomeEditError {
    #[error("Invalid Burst Engine Index {0}!")]
    UnknownBurstEngineIndex(BurstEngineIndex),
    #[error("Invalid cortical area Index {0}!")]
    UnknownCorticalAreaIndexSpecified(usize), // TODO arent usizes not allowed?
    #[error("Connectome Editing had an unknown failure. Connectome Corruption assumed")]
    UnknownFailure
}
 */
