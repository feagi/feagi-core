use core::future::Future;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::BurstEngineError;

#[cfg(feature = "engines-rayon")]
use feagi_npu_burst_rayon::rayon_burst_engine;

#[cfg(feature = "engines-esp32")]
use feagi_npu_burst_esp32::esp_32::burst_engine::ESP32BoardESP32BurstEngine;

// NOTE: for a constant API surface, keep this enum. In the case of a single variant, release mode
// will likely optimize this away anyways

/// Can contain a burst engine of composable or noncomposable type
pub enum EnclosedBurstEngine<FIQ: FeagiIndexQuantization> {
    #[cfg(feature = "alloc")]
    Composable(ComposableBurstEngine<FIQ>),
    NonComposable(NonComposableBurstEngine<FIQ>)
}

impl<FIQ: FeagiIndexQuantization> EnclosedBurstEngine<FIQ> {
    pub fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        match self {
            EnclosedBurstEngine::Composable(c) => {
                c.execute_phase(phases, burst_index)
            },
            EnclosedBurstEngine::NonComposable(n) => {
                n.execute_phase(phases, burst_index)
            }
        }
    }
}


//region Composable

#[cfg(feature = "alloc")]
pub enum ComposableBurstEngine<FIQ: FeagiIndexQuantization> {
    #[cfg(feature = "engines-rayon")]
    CPURayon(RayonBurstEngine<FIQ>),

    // Keeps `FIQ` live when every engine variant is cfg'd out.
    #[cfg(not(any(feature = "engines-rayon", feature = "engines-esp32")))]
    Impossible(core::marker::PhantomData<FIQ>), // Only here to get the compiler to shut up as it is blind to multi-feature setups
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> ComposableBurstEngine<FIQ> {
    #[inline(always)]
    pub fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        match self {
            #[cfg(feature = "engines-rayon")]
            ComposableBurstEngine::CPURayon(e) => e.execute_phase(phases, burst_index),
            #[cfg(not(any(feature = "engines-rayon")))]
            ComposableBurstEngine::Impossible(_) => async move {
                unreachable!("no composable burst engine feature is enabled")
            },
        }
    }

    // TODO Composition functions
}

//endregion

//region NonComposable

pub enum NonComposableBurstEngine<FIQ: FeagiIndexQuantization> {
    #[cfg(feature = "engines-esp32")]
    ESP32BoardESP32(ESP32BoardESP32BurstEngine<FIQ>),
    #[cfg(not(any(feature = "engines-esp32")))]
    Impossible(core::marker::PhantomData<FIQ>), // Only here to get the compiler to shut up as it is blind to multi-feature setups
}

impl<FIQ: FeagiIndexQuantization> NonComposableBurstEngine<FIQ> {
    #[inline(always)]
    pub fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        match self {
            #[cfg(feature = "engines-esp32")]
            NonComposableBurstEngine::ESP32BoardESP32(e) => e.execute_phase(phases, burst_index),
            #[cfg(not(any(feature = "engines-rayon", feature = "engines-esp32")))]
            NonComposableBurstEngine::Impossible(_) => async move {
                unreachable!("no non-composable burst engine feature is enabled")
            },
        }
    }
}

//endregion

/// Implemented on `BurstEngineSpawner` to easily make them a `EnclosedBurstEngine`
pub trait SpawnerToEnclosed<FIQ: FeagiIndexQuantization> {
    fn to_enclosed_burst_engine(self) -> EnclosedBurstEngine<FIQ>;
}

//region conversion trait impls

#[cfg(feature = "engines-rayon")]
impl<FIQ: FeagiIndexQuantization> SpawnerToEnclosed<FIQ> for RayonBurstEngine<FIQ> {
    fn to_enclosed_burst_engine(self) -> EnclosedBurstEngine<FIQ> {
        EnclosedBurstEngine::Composable(
            ComposableBurstEngine::CPURayon(self)
        )
    }
}


//endregion