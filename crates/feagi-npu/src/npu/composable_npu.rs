//! This is a true (composable) NPU, however it is currently being wrapped due to phase 1 work.
//! The job of the NPU is to handle timing, containing the async calls to the burst engine(s)

/// TODO We really shouldnt be using tokio this high up as its an optional feature, this would break wasm. It is to be noted that embedded wouldnt have composable npu anyways though

use futures::future::join_all;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::{interval, MissedTickBehavior};
use tokio_util::sync::CancellationToken;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine::burst_engine::BurstEngine;
use crate::burst_engine::composed_burst_engine::ComposableBurstEngine;
use crate::burst_engine_enum::ComposableBurstEngineEnum;




/// An NPU capable of editing the connectome it has loaded
pub struct ComposableNPU<FIQ: FeagiIndexQuantization + Send + 'static>
{
    engines: Vec<ComposableBurstEngineEnum<FIQ>>,
}

impl<FIQ: FeagiIndexQuantization + Send + 'static> ComposableNPU<FIQ>
{

    pub fn new() -> ComposableNPU<FIQ> {
        // TODO for now take no parameters and create a single tokio rayon Burst Engine.





        todo!()
    }

    pub fn load_connectome(&mut self) {

    }

    pub fn start_engine(&mut self, burst_frequency: BurstFrequency)
    {

    }

    pub fn stop_engine(&mut self)
    {

    }

    pub fn queue_connectome_request(&mut self) {

    }


    pub fn register_agent(&mut self)
    {
        todo!()
    }

    pub fn deregister_agent(&mut self)
    {
        todo!()
    }
}

// TODO make it take a generic of different enum types for composbale vs not, move this outside



struct BurstEngineRuntime<FIQ: FeagiIndexQuantization + Send + 'static> {
    burst_state: Option<BurstEngineRuntimeState<FIQ>>, // TODO is this the best rn?
}

impl<FIQ: FeagiIndexQuantization + Send + 'static> BurstEngineRuntime<FIQ>
{
    pub fn new(engines: Vec<ComposableBurstEngineEnum<FIQ>>) -> BurstEngineRuntime<FIQ>
    {
        BurstEngineRuntime { burst_state: Some(BurstEngineRuntimeState::NotRunningBurst(engines)) }
    }

    pub fn start(&mut self, frequency: BurstFrequency)
    {
        if !matches!(self.burst_state, Some(BurstEngineRuntimeState::NotRunningBurst(_))) {
            return;
        }

        let Some(BurstEngineRuntimeState::NotRunningBurst(mut inners)) = self.burst_state.take() else {
            panic!("Impossible!")
        };

        let cancel = CancellationToken::new();
        let child = cancel.child_token();
        let burst_period = Duration::from_secs_f64(1.0 / frequency);

        let handle = tokio::spawn(async move {

            let mut ticker = interval(burst_period);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Burst); // TODO pulled this from example, but this may not be desired

            loop {
                // Cancellation is observed ONLY here, while idle between ticks.
                tokio::select! {
                    _ = ticker.tick() => {}
                    _ = child.cancelled() => break, // TODO this may not be good, wouldnt this interrupt ongoing calculations and leave burst engines in unknown states? We should use a distinct "kill" to denote we are doing this
                }

                // If ended, only end once everyone is finished

                join_all(inners.iter_mut().map(
                    |i|
                        i.run_burst())
                ).await;
                
                if child.is_cancelled() {
                    break;
                }
            }

            return inners

        });

        self.burst_state = Some(BurstEngineRuntimeState::RunningBurst {cancel, handle});
    }



}



enum BurstEngineRuntimeState<FIQ: FeagiIndexQuantization> {
    NotRunningBurst(Vec<ComposableBurstEngineEnum<FIQ>>),
    RunningBurst{
        cancel: CancellationToken,
        handle: JoinHandle<Vec<ComposableBurstEngineEnum<FIQ>>>
    }    // TODO separation between paused and awaiting next burst
}
