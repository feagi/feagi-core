use crate::quantization_levels::FeagiIndexQuantization;

// TODO Edits???

pub struct EngineChangeCommand<FIQ: FeagiIndexQuantization>(core::marker::PhantomData<FIQ>);

pub struct EngineChangeCommandReturn<FIQ: FeagiIndexQuantization>(core::marker::PhantomData<FIQ>);

#[derive(Debug, thiserror::Error)]
pub enum EngineChangeCommandError {
    #[error("Unknown Error")]
    UnknownError
}