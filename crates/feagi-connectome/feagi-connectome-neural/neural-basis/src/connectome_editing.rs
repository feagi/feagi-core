use crate::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use crate::quantization_levels::FeagiIndexQuantization;
use crate::visualization::NeuronVisualizationWordIndex;

macro_rules! _impl_connectome_edit_step_requirements {
    {$StepSpecificName:ident} => {
        paste::paste! {



            impl<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> ConnectomeEditStepType<FIQ, CVDW> for [<ConnectomeEditStep $StepSpecificName>]<FIQ, CVDW> {
                type ConnectomeEditResultant = [<ConnectomeEditResultant $StepSpecificName>]<FIQ>;
                type ConnectomeEditError = [<ConnectomeEditError $StepSpecificName>];
            }

            impl<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> From<[<ConnectomeEditStep $StepSpecificName>]<FIQ, CVDW>> for ConnectomeEditStepEnum<FIQ, CVDW> {
                fn from(value: [<ConnectomeEditStep $StepSpecificName>]<FIQ, CVDW>) -> Self {
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

macro_rules! create_step_and_result_enums {
    {
        $($StepSpecificName:ident, )+
    } => {

        paste::paste! {

            pub enum ConnectomeEditStepEnum<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> {
                /// Used by GenomeEditStep to skip over edits that dont affect the connectome
                Nothing,
                $($StepSpecificName([<ConnectomeEditStep $StepSpecificName>]<FIQ, CVDW>),)+
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

pub trait ConnectomeVisitorDispatchWrapper<FIQ: FeagiIndexQuantization> {
    type CorticalAreaNeurogenesisDispatch: Send;
}

/// Defines some sort of edit step for the connectome, and the result that should be returned
pub trait ConnectomeEditStepType<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>>:
Sized + Into<ConnectomeEditStepEnum<FIQ, CVDW>> {
    /// What would be returned if everything went fine
    type ConnectomeEditResultant: Into<ConnectomeEditResultantEnum<FIQ>>;
    /// What is returned if something when wrong
    type ConnectomeEditError: ConnectomeEditError + Into<ConnectomeEditResultantEnum<FIQ>>;
}


pub trait ConnectomeEditError: core::error::Error{
    /// Return true if the error likely caused connectome corruption
    fn is_connectome_corrupted(&self) -> bool;
}


/// Generate `ConnectomeEditStepEnum` and `ConnectomeEditResultantEnum`
create_step_and_result_enums!{
    CorticalAreaAdd,
}

// TODO -> Edit genome-prop-inplace, Variance,  Delete, Resize



//region Cortical Areas

pub struct ConnectomeEditStepCorticalAreaAdd<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> {
    pub cortical_area_neurogenesis_dispatch: CVDW::CorticalAreaNeurogenesisDispatch,
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> ConnectomeEditStepCorticalAreaAdd<FIQ, CVDW> {
    /// Adds a cortical area from its neurogenesis, at whichever neuron model it uses
    pub fn new(cortical_area_neurogenesis_dispatch: CVDW::CorticalAreaNeurogenesisDispatch) -> Self {
        Self {
            cortical_area_neurogenesis_dispatch,
            _p: core::marker::PhantomData,
        }
    }
}

pub struct ConnectomeEditResultantCorticalAreaAdd<FIQ: FeagiIndexQuantization>
{
    pub new_cortical_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub visualization_start_index: NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>
}

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
