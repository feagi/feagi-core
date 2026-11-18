//! Connectome Reshaping is a subset of connectome edits, and involve edits which add / remove
//! data from the connectome. Ergo, alloc access is required.

use connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use connectome_basis::visualization::NeuronVisualizationVoxelByteIndex;
use connectome_models::neuron_models::models::NMNeurogenesisStorageOrGenerator;

/// Implements `ConnectomeReshapeStep`, `ConnectomeReshapeStepResultant`, and the conversions into
/// `ConnectomeReshapeStepEnum` / `ConnectomeReshapeStepResultantEnum` for a single step.
/// The optional `<FIQ>` marks that the step and resultant structs are generic over FIQ
macro_rules! _impl_connectome_reshape_step_requirements {
    {$StepSpecificName:ident $(<$Generic:ident>)?} => {
        paste::paste! {

            impl<FIQ: FeagiIndexQuantization> ConnectomeReshapeStep<FIQ> for [<ConnectomeReshapeStep $StepSpecificName>]$(<$Generic>)? {
                type Resultant = [<ConnectomeReshapeResultant $StepSpecificName>]$(<$Generic>)?;
            }

            impl<FIQ: FeagiIndexQuantization> ConnectomeReshapeStepResultant<FIQ> for [<ConnectomeReshapeResultant $StepSpecificName>]$(<$Generic>)? {}

            impl<FIQ: FeagiIndexQuantization> From<[<ConnectomeReshapeStep $StepSpecificName>]$(<$Generic>)?> for ConnectomeReshapeStepEnum<FIQ> {
                fn from(value: [<ConnectomeReshapeStep $StepSpecificName>]$(<$Generic>)?) -> Self {
                    ConnectomeReshapeStepEnum::$StepSpecificName(value)
                }
            }

            impl<FIQ: FeagiIndexQuantization> From<[<ConnectomeReshapeResultant $StepSpecificName>]$(<$Generic>)?> for ConnectomeReshapeStepResultantEnum<FIQ> {
                fn from(value: [<ConnectomeReshapeResultant $StepSpecificName>]$(<$Generic>)?) -> Self {
                    ConnectomeReshapeStepResultantEnum::$StepSpecificName(value)
                }
            }
        }
    };
}

/// Creates ConnectomeReshapeStepEnum<FIQ: FeagiIndexQuantization>,
/// ConnectomeReshapeStepResultantEnum<FIQ: FeagiIndexQuantization>,
/// ConnectomeReshapeStepEnumHandler<FIQ: FeagiIndexQuantization>
macro_rules! create_step_and_result_enums {
    {
        $($StepSpecificName:ident $(<$Generic:ident>)?, )+
    } => {

        paste::paste! {
            
            pub enum ConnectomeReshapeStepEnum<FIQ: FeagiIndexQuantization> {
                $($StepSpecificName([<ConnectomeReshapeStep $StepSpecificName>]$(<$Generic>)?),)+
            }

            #[derive(Debug)]
            pub enum ConnectomeReshapeStepResultantEnum<FIQ: FeagiIndexQuantization> {
                $($StepSpecificName([<ConnectomeReshapeResultant $StepSpecificName>]$(<$Generic>)?),)+
            }

            /// Implemented by burst engines that can handle every reshape step
            pub trait ConnectomeReshapeStepEnumHandler<FIQ: FeagiIndexQuantization>:
            Sized
            $(+ ConnectomeReshapeStepHandler<FIQ, [<ConnectomeReshapeStep $StepSpecificName>]$(<$Generic>)?>)+
            {
                async fn handle_connectome_reshape_step_enum(&mut self, step: ConnectomeReshapeStepEnum<FIQ>)
                    -> Result<ConnectomeReshapeStepResultantEnum<FIQ>, ConnectomeReshapeError<FIQ>> {

                    match step {
                        $(
                            ConnectomeReshapeStepEnum::$StepSpecificName(step) => {
                                <Self as ConnectomeReshapeStepHandler<FIQ, [<ConnectomeReshapeStep $StepSpecificName>]$(<$Generic>)?>>
                                    ::handle_reshape(self, step).await.map(Into::into)
                            }
                        )+
                    }
                }
            }

            $( _impl_connectome_reshape_step_requirements!($StepSpecificName $(<$Generic>)?); )+

        }

    }
}

/// Defines some sort of reshape step for the connectome, and the result that should be returned
pub trait ConnectomeReshapeStep<FIQ: FeagiIndexQuantization>: Sized + Into<ConnectomeReshapeStepEnum<FIQ>> {
    /// What would be returned if everything went fine
    type Resultant: ConnectomeReshapeStepResultant<FIQ>;
}

/// What an reshape step returns
pub trait ConnectomeReshapeStepResultant<FIQ: FeagiIndexQuantization>: Sized + Into<ConnectomeReshapeStepResultantEnum<FIQ>> {}

/// A variant of this is implemented for every reshape step on an reshapeable burst engine
pub trait ConnectomeReshapeStepHandler<FIQ: FeagiIndexQuantization, Reshape: ConnectomeReshapeStep<FIQ>> {
    /// Handle running the reshape
    async fn handle_reshape(&mut self, step: Reshape) -> Result<Reshape::Resultant, ConnectomeReshapeError<FIQ>>;
}

// Generate `ConnectomeReshapeStepEnum`, `ConnectomeReshapeStepResultantEnum`, `ConnectomeReshapeStepEnumHandler`,
// and the trait / conversion impls for each step. Append `<FIQ>` if the step structs are generic over FIQ.
create_step_and_result_enums! {
    CorticalAreaAdd<FIQ>,
}

//region Steps

//region Nothing

//endregion

//region Cortical Areas

//region Add

pub struct ConnectomeReshapeStepCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
    pub cortical_area_neurogenesis: NMNeurogenesisStorageOrGenerator<FIQ>,
}

#[derive(Debug)]
pub struct ConnectomeReshapeResultantCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
    pub new_cortical_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub visualization_start_index: NeuronVisualizationVoxelByteIndex<FIQ::NeuronIndexQuant>,
}

//endregion

// TODO DELETE

// TODO RESIZE

//endregion

//region Cortical Mappings

// TODO ADD

// TODO DELETE

//endregion

//endregion

#[derive(Debug, thiserror::Error)]
pub enum ConnectomeReshapeError<FIQ: FeagiIndexQuantization> {
    #[error("Unknown Failure. Connectome Corruption assumed")]
    UnknownFailure,
    #[error("Cortical area index {0} is invalid and cannot be selected for this operation")]
    InvalidCorticalAreaIndex(CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>),
    #[error("A computation exceeding possible quantized indexing limits was attempted!")]
    ExceedsIndexQuantization,
}

impl<FIQ: FeagiIndexQuantization> ConnectomeReshapeError<FIQ> {
    pub fn is_corrupted(&self) -> bool {
        match &self {
            ConnectomeReshapeError::UnknownFailure => true,
            ConnectomeReshapeError::ExceedsIndexQuantization => true, // if this happens, something went VERY wrong
            _ => false,
        }
    }
}
