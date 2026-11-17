use connectome_basis::quantization_levels::FeagiIndexQuantization;

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
{
    async fn handle_burst_engine_command_enum(&mut self, command: BurstEngineCommandEnum<FIQ>) -> Result<BurstEngineCommandResultantEnum<FIQ>, BurstEngineCommandError<FIQ>>
    {
        match command {
            BurstEngineCommandEnum::Nothing(a) => {self.handle_command(a)}

            #[cfg(feature = "allow-edit-commands")]
            BurstEngineCommandEnum::EditConnectome() => {self.handle_command(a)}
        }
    }
}

/// Commands relevant to a specific burst engine as a passable enum
pub enum BurstEngineCommandEnum<FIQ: FeagiIndexQuantization> {
    Nothing(BurstEngineCommandNothing),
    // TODO more (in memory stuff)
    #[cfg(feature = "allow-edit-commands")]
    EditConnectome()
}

pub enum BurstEngineCommandResultantEnum<FIQ: FeagiIndexQuantization> {
    Nothing(BurstEngineCommandResultantNothing),
    #[cfg(feature = "allow-edit-commands")]
    EditConnectome()
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



//endregion



#[derive(Debug, thiserror::Error)]
pub enum BurstEngineCommandError<FIQ: FeagiIndexQuantization> {
    #[error("Burst Engine has failed! {0}")]
    GeneralEngineFailure(&'static str, bool), // second bool indicates if corrupt

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

            #[cfg(not(feature = "allow-edit-commands"))]
            BurstEngineCommandError::ConnectomeEditingNotAllowed => false
        }
    }
}
