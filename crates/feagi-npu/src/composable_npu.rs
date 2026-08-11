//! This is a true (composable) NPU, however it is currently being wrapped due to phase 1 work.
//! The job of the NPU is to handle timing, containing the async calls to the burst engine(s)

use futures::future::join_all;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::{interval, MissedTickBehavior};
use tokio_util::sync::CancellationToken;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine::burst_engine::BurstEngine;
use crate::burst_engine::composed_burst_engine::ComposableBurstEngine;
use crate::burst_engine_enum::ComposableBurstEngineEnum;

/// Number of bursts per second
pub type BurstFrequency = f64;


/// An NPU capable of editing the connectome it has loaded
pub struct ComposableNPU<FIQ: FeagiIndexQuantization>
{
    engines: Vec<ComposableBurstEngineEnum<FIQ>>,
}

impl<FIQ: FeagiIndexQuantization> ComposableNPU<FIQ>
{

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



struct BurstEngineRuntime<FIQ: FeagiIndexQuantization> {
    burst_state: Option<BurstEngineRuntimeState<FIQ>>, // TODO is this the best rn?
}

impl<FIQ: FeagiIndexQuantization> BurstEngineRuntime<FIQ>
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
                    _ = child.cancelled() => break,
                }

                // If ended, only end once everyone is finished

                join_all(inners.iter_mut().map(|i| i.run_bursts())).await;
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
