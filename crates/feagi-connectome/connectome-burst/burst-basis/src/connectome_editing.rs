use connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use connectome_basis::visualization::NeuronVisualizationVoxelByteIndex;
use connectome_models::neuron_models::models::NMNeurogenesisStorageOrGenerator;

/// Implements `ConnectomeEditStep`, `ConnectomeEditStepResultant`, and the conversions into
/// `ConnectomeEditStepEnum` / `ConnectomeEditStepResultantEnum` for a single step.
/// The optional `<FIQ>` marks that the step and resultant structs are generic over FIQ
macro_rules! _impl_connectome_edit_step_requirements {
    {$StepSpecificName:ident $(<$Generic:ident>)?} => {
        paste::paste! {

            impl<FIQ: FeagiIndexQuantization> ConnectomeEditStep<FIQ> for [<ConnectomeEditStep $StepSpecificName>]$(<$Generic>)? {
                type Resultant = [<ConnectomeEditResultant $StepSpecificName>]$(<$Generic>)?;
            }

            impl<FIQ: FeagiIndexQuantization> ConnectomeEditStepResultant<FIQ> for [<ConnectomeEditResultant $StepSpecificName>]$(<$Generic>)? {}

            impl<FIQ: FeagiIndexQuantization> From<[<ConnectomeEditStep $StepSpecificName>]$(<$Generic>)?> for ConnectomeEditStepEnum<FIQ> {
                fn from(value: [<ConnectomeEditStep $StepSpecificName>]$(<$Generic>)?) -> Self {
                    ConnectomeEditStepEnum::$StepSpecificName(value)
                }
            }

            impl<FIQ: FeagiIndexQuantization> From<[<ConnectomeEditResultant $StepSpecificName>]$(<$Generic>)?> for ConnectomeEditStepResultantEnum<FIQ> {
                fn from(value: [<ConnectomeEditResultant $StepSpecificName>]$(<$Generic>)?) -> Self {
                    ConnectomeEditStepResultantEnum::$StepSpecificName(value)
                }
            }
        }
    };
}

/// Creates ConnectomeEditStepEnum<FIQ: FeagiIndexQuantization>,
/// ConnectomeEditStepResultantEnum<FIQ: FeagiIndexQuantization>,
/// ConnectomeEditStepEnumHandler<FIQ: FeagiIndexQuantization>
macro_rules! create_step_and_result_enums {
    {
        $($StepSpecificName:ident $(<$Generic:ident>)?, )+
    } => {

        paste::paste! {

            pub enum ConnectomeEditStepEnum<FIQ: FeagiIndexQuantization> {
                $($StepSpecificName([<ConnectomeEditStep $StepSpecificName>]$(<$Generic>)?),)+
            }

            pub enum ConnectomeEditStepResultantEnum<FIQ: FeagiIndexQuantization> {
                $($StepSpecificName([<ConnectomeEditResultant $StepSpecificName>]$(<$Generic>)?),)+
            }

            /// Implemented by burst engines that can handle every edit step
            pub trait ConnectomeEditStepEnumHandler<FIQ: FeagiIndexQuantization>:
            Sized
            $(+ ConnectomeEditStepHandler<FIQ, [<ConnectomeEditStep $StepSpecificName>]$(<$Generic>)?>)+
            {
                async fn handle_connectome_edit_step_enum(&mut self, step: ConnectomeEditStepEnum<FIQ>)
                    -> Result<ConnectomeEditStepResultantEnum<FIQ>, ConnectomeEditError<FIQ>> {

                    match step {
                        $(
                            ConnectomeEditStepEnum::$StepSpecificName(step) => {
                                <Self as ConnectomeEditStepHandler<FIQ, [<ConnectomeEditStep $StepSpecificName>]$(<$Generic>)?>>
                                    ::handle_edit(self, step).await.map(Into::into)
                            }
                        )+
                    }
                }
            }

            $( _impl_connectome_edit_step_requirements!($StepSpecificName $(<$Generic>)?); )+

        }

    }
}

/// Defines some sort of edit step for the connectome, and the result that should be returned
pub trait ConnectomeEditStep<FIQ: FeagiIndexQuantization>: Sized + Into<ConnectomeEditStepEnum<FIQ>> {
    /// What would be returned if everything went fine
    type Resultant: ConnectomeEditStepResultant<FIQ>;
}

/// What an edit step returns
pub trait ConnectomeEditStepResultant<FIQ: FeagiIndexQuantization>: Sized + Into<ConnectomeEditStepResultantEnum<FIQ>> {}

/// A variant of this is implemented for every edit step on an editable burst engine
pub trait ConnectomeEditStepHandler<FIQ: FeagiIndexQuantization, Edit: ConnectomeEditStep<FIQ>> {
    /// Handle running the edit
    async fn handle_edit(&mut self, step: Edit) -> Result<Edit::Resultant, ConnectomeEditError<FIQ>>;
}

// Generate `ConnectomeEditStepEnum`, `ConnectomeEditStepResultantEnum`, `ConnectomeEditStepEnumHandler`,
// and the trait / conversion impls for each step. Append `<FIQ>` if the step structs are generic over FIQ.
create_step_and_result_enums! {
    Nothing,
    CorticalAreaAdd<FIQ>,
}

//region Steps

//region Nothing

pub struct ConnectomeEditStepNothing;

pub struct ConnectomeEditResultantNothing;

//endregion

//region Cortical Areas

//region Add

pub struct ConnectomeEditStepCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
    pub cortical_area_neurogenesis: NMNeurogenesisStorageOrGenerator<FIQ>,
}

pub struct ConnectomeEditResultantCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
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
pub enum ConnectomeEditError<FIQ: FeagiIndexQuantization> {
    #[error("Unknown Failure. Connectome Corruption assumed")]
    UnknownFailure,
    #[error("Cortical area index {0} is invalid and cannot be selected for this operation")]
    InvalidCorticalAreaIndex(CorticalAreaEngineIndex<FIQ>),
    #[error("A computation exceeding possible quantized indexing limits was attempted!")]
    ExceedsIndexQuantization,
}

impl<FIQ: FeagiIndexQuantization> ConnectomeEditError<FIQ> {
    pub fn is_corrupted(&self) -> bool {
        match &self {
            ConnectomeEditError::UnknownFailure => true,
            ConnectomeEditError::ExceedsIndexQuantization => true, // if this happens, something went VERY wrong
            _ => false,
        }
    }
}
