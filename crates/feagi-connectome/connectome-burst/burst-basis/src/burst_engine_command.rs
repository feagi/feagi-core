use connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::connectome_editing::{ConnectomeEditError, ConnectomeEditStepEnum, ConnectomeEditStepResultantEnum};
//region Root Traits

/// Commands relevant to a specific burst engine
pub trait BurstEngineCommand<FIQ: FeagiIndexQuantization>:
Sized + Into<BurstEngineCommandEnum<FIQ>> {
    /// What the response must be
    type Resultant: BurstEngineCommandResultant<FIQ>;
}

/// A non-error response for a given command
pub trait BurstEngineCommandResultant<FIQ: FeagiIndexQuantization>:
Sized + Into<BurstEngineCommandResultantEnum<FIQ>> {}

/// Handling a specific command to get a response or an error
pub trait BurstEngineCommandHandler<FIQ: FeagiIndexQuantization, Command: BurstEngineCommand<FIQ>> {
    /// Burst engine tries to process the request
    async fn handle_command(&mut self, command: Command) -> Result<Command::Resultant, BurstEngineCommandError<FIQ>>;
}

//endregion

//region Grouped via Enums

pub trait BurstEngineCommandEnumHandler<FIQ: FeagiIndexQuantization>:
Sized
+ BurstEngineCommandHandler<FIQ, BurstEngineCommandNothing>
+ BurstEngineCommandHandler<FIQ, BurstEngineCommandConnectomeEditStep<FIQ>>
{
    async fn handle_burst_engine_command_enum(&mut self, command: BurstEngineCommandEnum<FIQ>) -> Result<BurstEngineCommandResultantEnum<FIQ>, BurstEngineCommandError<FIQ>>
    {
        match command {
            BurstEngineCommandEnum::Nothing(a) => {self.handle_command(a)}

            #[cfg(feature = "allow-edit-commands")]
            BurstEngineCommandEnum::EditConnectomeStep(a) => {self.handle_command(a)}
        }
    }
}

/// Commands relevant to a specific burst engine as a passable enum
pub enum BurstEngineCommandEnum<FIQ: FeagiIndexQuantization> {
    Nothing(BurstEngineCommandNothing),
    // TODO more (in memory stuff)
    #[cfg(feature = "allow-edit-commands")]
    EditConnectomeStep(BurstEngineCommandConnectomeEditStep<FIQ>)
}

pub enum BurstEngineCommandResultantEnum<FIQ: FeagiIndexQuantization> {
    Nothing(BurstEngineCommandResultantNothing),
    #[cfg(feature = "allow-edit-commands")]
    EditConnectomeStep(BurstEngineCommandResultantConnectomeEditStepResultant<FIQ>)
}

//endregion

//region Commands And Responses

//region Nothing

/// Does nothing. Effectively a placeholder for some things
pub struct BurstEngineCommandNothing;

impl<FIQ: FeagiIndexQuantization> BurstEngineCommand<FIQ> for BurstEngineCommandNothing {
    type Resultant = BurstEngineCommandResultantNothing;
}

impl<FIQ: FeagiIndexQuantization> Into<BurstEngineCommandEnum<FIQ>> for BurstEngineCommandNothing {
    fn into(self) -> BurstEngineCommandEnum<FIQ> { BurstEngineCommandEnum::Nothing(self) }
}

/// Response to a Nothing Command. No possible error
pub struct BurstEngineCommandResultantNothing;

impl<FIQ: FeagiIndexQuantization> BurstEngineCommandResultant<FIQ> for BurstEngineCommandResultantNothing {}

impl<FIQ: FeagiIndexQuantization> Into<BurstEngineCommandResultantEnum<FIQ>> for BurstEngineCommandResultantNothing {
    fn into(self) -> BurstEngineCommandResultantEnum<FIQ> { BurstEngineCommandResultantEnum::Nothing(self) }
}

//endregion

// TODO Edit in place

// TODO Force Fire

// TODO Probes

//region Connectome Edit

// NOTE: ConnectomeEditStepEnum<FIQ> if from another mod, as is ConnectomeEditStepResultantEnum<FIQ

type BurstEngineCommandConnectomeEditStep<FIQ: FeagiIndexQuantization> = ConnectomeEditStepEnum<FIQ>;

impl<FIQ: FeagiIndexQuantization> BurstEngineCommand<FIQ> for BurstEngineCommandConnectomeEditStep<FIQ> {
    type Resultant = BurstEngineCommandResultantConnectomeEditStepResultant<FIQ>;
}

impl<FIQ: FeagiIndexQuantization> Into<BurstEngineCommandEnum<FIQ>> for BurstEngineCommandConnectomeEditStep<FIQ> {
    fn into(self) -> BurstEngineCommandEnum<FIQ> { BurstEngineCommandEnum::EditConnectomeStep(self) }
}

type BurstEngineCommandResultantConnectomeEditStepResultant<FIQ: FeagiIndexQuantization> = ConnectomeEditStepResultantEnum<FIQ>;

impl<FIQ: FeagiIndexQuantization> BurstEngineCommandResultant<FIQ> for ConnectomeEditStepResultantEnum<FIQ> {}

impl<FIQ: FeagiIndexQuantization> Into<BurstEngineCommandResultantEnum<FIQ>> for ConnectomeEditStepResultantEnum<FIQ> {
    fn into(self) -> BurstEngineCommandResultantEnum<FIQ> { BurstEngineCommandResultantEnum::EditConnectomeStep(self) }
}

//endregion


//endregion



#[derive(Debug, thiserror::Error)]
pub enum BurstEngineCommandError<FIQ: FeagiIndexQuantization> {
    #[error("Burst Engine has failed! {0}")]
    GeneralEngineFailure(&'static str, bool), // second bool indicates if corrupt
    #[error("Cortical area index {0} is invalid and cannot be read for this operation")]
    InvalidCorticalAreaIndex(CorticalAreaEngineIndex<FIQ>),

    #[cfg(feature = "allow-edit-commands")]
    #[error(transparent)]
    ConnectomeEditingError {
        completed_steps: Vec<ConnectomeEditStepResultantEnum<FIQ>>,
        error: ConnectomeEditError<FIQ>
    },
    #[cfg(not(feature = "allow-edit-commands"))]
    /// This is to be returned on traits implementations where editing may be disabled
    #[error("Burst Engine has received a connectome edit request, but this feature is disabled! Ignoring!")]
    ConnectomeEditingNotAllowed
}


impl<FIQ: FeagiIndexQuantization> BurstEngineCommandError<FIQ> {
    /// Returns true if connectome is corrupted (IE not resumable)
    pub fn is_connectome_corrupted(&self) -> bool {
        match &self {
            BurstEngineCommandError::GeneralEngineFailure(_, c) => {*c}
            BurstEngineCommandError::InvalidCorticalAreaIndex(i) => {false}

            #[cfg(feature = "allow-edit-commands")]
            BurstEngineCommandError::ConnectomeEditingError {error, .. } => {error.is_corrupted()}

            #[cfg(not(feature = "allow-edit-commands"))]
            BurstEngineCommandError::ConnectomeEditingNotAllowed => false
        }
    }
}
