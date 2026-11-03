use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


pub struct EngineChangeCommand<FIQ: FeagiIndexQuantization>(core::marker::PhantomData<FIQ>);

pub struct EngineChangeCommandReturn<FIQ: FeagiIndexQuantization>(core::marker::PhantomData<FIQ>);

#[derive(Debug, thiserror::Error)]
pub enum EngineChangeCommandError {
    #[error("Unknown Error")]
    UnknownError
}