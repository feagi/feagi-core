//! Connectome Editing allows for changing the state of the connectome from the outside. This
//! includes anything from just forcing firing some neurons to editing genome properties to 
//! adding / removing cortical areas (these types of edits are considered "reshaping" and require
//! the flag for this to be enabled and requires alloc).

use connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::connectome_reshaping::{ConnectomeReshapeStepEnum, ConnectomeReshapeStepResultantEnum};

//region Root Traits

/// Commands relevant to a specific burst engine
pub trait ConnectomeEditStep<FIQ: FeagiIndexQuantization>: Sized + Into<ConnectomeEditEnum<FIQ>> {
    /// What the response must be
    type Resultant: ConnectomeEditResultant<FIQ>;
}

/// A non-error response for a given edit
pub trait ConnectomeEditResultant<FIQ: FeagiIndexQuantization>: Sized + Into<ConnectomeEditResultantEnum<FIQ>> {}

/// Handling a specific edit to get a response or an error
pub trait ConnectomeEditHandler<FIQ: FeagiIndexQuantization, Command: ConnectomeEditStep<FIQ>> {
    /// Burst engine tries to process the request
    async fn handle_edit(&mut self, edit: Command) -> Result<Command::Resultant, ConnectomeEditError<FIQ>>;
}

//endregion

//region Grouped via Enums

pub trait ConnectomeEditEnumHandler<FIQ: FeagiIndexQuantization>:
    Sized + ConnectomeEditHandler<FIQ, ConnectomeEditNothing>
+ ConnectomeEditHandler<FIQ, ConnectomeEditReshapeStep<FIQ>>
{
    /// Has the burst engine run some edit
    async fn handle_connectome_edit_enum(
        &mut self,
        edit: ConnectomeEditEnum<FIQ>,
    ) -> Result<ConnectomeEditResultantEnum<FIQ>, ConnectomeEditError<FIQ>> {
        match edit {
            ConnectomeEditEnum::Nothing(a) => self.handle_edit(a),

            #[cfg(feature = "allow-reshape-edits")]
            ConnectomeEditEnum::EditConnectomeStep(a) => self.handle_edit(a),
        }
    }
}

/// Commands relevant to a specific burst engine as a passable enum
pub enum ConnectomeEditEnum<FIQ: FeagiIndexQuantization> {
    Nothing(ConnectomeEditNothing),
    // TODO more (in memory stuff)
    #[cfg(feature = "allow-reshape-edits")]
    EditConnectomeStep(ConnectomeEditReshapeStep<FIQ>),
}

pub enum ConnectomeEditResultantEnum<FIQ: FeagiIndexQuantization> {
    Nothing(ConnectomeEditResultantNothing),
    #[cfg(feature = "allow-reshape-edits")]
    EditConnectomeStep(ConnectomeEditReshapeStepResultant<FIQ>),
}

//endregion

//region Edits and Resulants

//region Nothing

/// Does nothing. Effectively a placeholder for some things
pub struct ConnectomeEditNothing;

impl<FIQ: FeagiIndexQuantization> ConnectomeEditStep<FIQ> for ConnectomeEditNothing {
    type Resultant = ConnectomeEditResultantNothing;
}

impl<FIQ: FeagiIndexQuantization> Into<ConnectomeEditEnum<FIQ>> for ConnectomeEditNothing {
    fn into(self) -> ConnectomeEditEnum<FIQ> {
        ConnectomeEditEnum::Nothing(self)
    }
}

/// Response to a Nothing Command. No possible error
pub struct ConnectomeEditResultantNothing;

impl<FIQ: FeagiIndexQuantization> ConnectomeEditResultant<FIQ> for ConnectomeEditResultantNothing {}

impl<FIQ: FeagiIndexQuantization> Into<ConnectomeEditResultantEnum<FIQ>> for ConnectomeEditResultantNothing {
    fn into(self) -> ConnectomeEditResultantEnum<FIQ> {
        ConnectomeEditResultantEnum::Nothing(self)
    }
}

//endregion

// TODO Edit in place

// TODO Force Fire

// TODO Probes

//region Connectome Reshaping

// NOTE: ConnectomeReshapeStepEnum<FIQ> if from another mod, as is ConnectomeEditStepResultantEnum<FIQ

type ConnectomeEditReshapeStep<FIQ: FeagiIndexQuantization> = ConnectomeReshapeStepEnum<FIQ>;

impl<FIQ: FeagiIndexQuantization> ConnectomeEditStep<FIQ> for ConnectomeEditReshapeStep<FIQ> {
    type Resultant = ConnectomeEditReshapeStepResultant<FIQ>;
}

impl<FIQ: FeagiIndexQuantization> Into<ConnectomeEditEnum<FIQ>> for ConnectomeEditReshapeStep<FIQ> {
    fn into(self) -> ConnectomeEditEnum<FIQ> {
        ConnectomeEditEnum::EditConnectomeStep(self)
    }
}

type ConnectomeEditReshapeStepResultant<FIQ: FeagiIndexQuantization> = ConnectomeReshapeStepResultantEnum<FIQ>;

impl<FIQ: FeagiIndexQuantization> ConnectomeEditResultant<FIQ> for ConnectomeReshapeStepResultantEnum<FIQ> {}

impl<FIQ: FeagiIndexQuantization> Into<ConnectomeEditResultantEnum<FIQ>> for ConnectomeReshapeStepResultantEnum<FIQ> {
    fn into(self) -> ConnectomeEditResultantEnum<FIQ> {
        ConnectomeEditResultantEnum::EditConnectomeStep(self)
    }
}

//endregion

//endregion

#[derive(Debug, thiserror::Error)]
pub enum ConnectomeEditError<FIQ: FeagiIndexQuantization> {
    #[error("Burst Engine has failed! {0}")]
    GeneralEngineFailure(&'static str, bool), // second bool indicates if corrupt
    #[error("Cortical area index {0} is invalid and cannot be read for this operation")]
    InvalidCorticalAreaIndex(CorticalAreaEngineIndex<FIQ>),

    #[cfg(feature = "allow-reshape-edits")]
    #[error(transparent)]
    ConnectomeReshapingError {
        completed_steps: Vec<ConnectomeReshapeStepResultantEnum<FIQ>>,
        error: ConnectomeEditError<FIQ>,
    },
    #[cfg(not(feature = "allow-reshape-edits"))]
    /// This is to be returned on traits implementations where editing may be disabled
    #[error("Burst Engine has received a connectome edit request, but this feature is disabled! Ignoring!")]
    ConnectomeEditingNotAllowed,
}

impl<FIQ: FeagiIndexQuantization> ConnectomeEditError<FIQ> {
    /// Returns true if connectome is corrupted (IE not resumable)
    pub fn is_connectome_corrupted(&self) -> bool {
        match &self {
            ConnectomeEditError::GeneralEngineFailure(_, c) => *c,
            ConnectomeEditError::InvalidCorticalAreaIndex(i) => false,

            #[cfg(feature = "allow-reshape-edits")]
            ConnectomeEditError::ConnectomeReshapingError { error, .. } => error.is_corrupted(),

            #[cfg(not(feature = "allow-reshape-edits"))]
            ConnectomeEditError::ConnectomeEditingNotAllowed => false,
        }
    }
}
