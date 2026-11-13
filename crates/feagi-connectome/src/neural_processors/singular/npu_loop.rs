use crate::connectome_basis::connectome_editing::ConnectomeDispatch;
use crate::neural_processors::singular::commands::NeuronProcessorSingleEngineCommandHandler;
use crate::neural_processors::singular::commands::{NeuronProcessorSingleEngineRequest, NeuronProcessorSingleEngineResponse};
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiResponderAlloc;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::FeagiWorkerCommandRouter;
use feagi_connectome_neural::neural_basis::engine::BurstEngine;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use std::ops::ControlFlow;
use std::time::{Duration, Instant};

/// The burst loop. Each tick: schedules the next tick, handles every queued command, runs the
/// burst (sending visualization if it is due), then waits until the scheduled tick. If a command
/// paused it, it instead waits on and handles commands until resumed, then bursts
pub(crate) async fn run_neuron_processor_single_engine<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ConnectomeDispatch, ()>>(
    responder: FeagiResponderAlloc<
        FeagiWorkerRequest<NeuronProcessorSingleEngineRequest<FIQ>>,
        FeagiWorkerResponse<NeuronProcessorSingleEngineResponse<FIQ>>,
    >,
    handler: NeuronProcessorSingleEngineCommandHandler<FIQ, Engine>,
) {
    let mut command_router = FeagiWorkerCommandRouter::new(handler, responder);
    let mut tick_time = Instant::now();
    let mut visualization_time = Instant::now();

    loop {
        // Scheduled from the previous tick so the rate does not drift. If the previous tick ran
        // over, schedule from now instead of bursting to catch up
        let tick_duration = command_router.processor_mut().target_frequency.duration_between_ticks();
        tick_time = (tick_time + tick_duration).max(Instant::now());

        loop {
            match command_router.try_handle_next().await {
                // processes command (if exist)
                ControlFlow::Continue(true) => continue, // more commands in queue
                ControlFlow::Continue(false) => break,   // no more commands in queue
                ControlFlow::Break(_) => return,         // channel killbinded
            }
        }

        // While paused, nothing runs on the timer. The worker waits for commands and handles each
        // one as it arrives, without spending time between them
        if command_router.processor_mut().paused {
            while command_router.processor_mut().paused {
                if command_router.await_handle_next().await.is_break() {
                    return;
                }
            }
            // Resumed. Burst now, and schedule the next tick from now rather than catching up
            let tick_duration = command_router.processor_mut().target_frequency.duration_between_ticks();
            tick_time = Instant::now() + tick_duration;
        }

        // Visualization is scheduled the same way as ticks, but only checked once per burst
        let now = Instant::now();
        let send_visualization = now >= visualization_time;
        if send_visualization {
            let visualization_duration = command_router.processor_mut().visualization_frequency.duration_between_ticks();
            visualization_time = (visualization_time + visualization_duration).max(now);
        }

        command_router.processor_mut().engine.run_burst(send_visualization).await;

        wait_until(tick_time).await;
    }
}

// TODO use Futures to delay instead, we dont want to std sleep in async

/// How long before a tick `wait_until` stops sleeping and starts spinning. Uses normal
/// OS sleep but swaps to hot-spin when the timing gets close
const SPIN_MARGIN: Duration = Duration::from_millis(1);

/// Waits until `deadline` by sleeping until `SPIN_MARGIN` before it, then spinning the rest. The
/// worker has its own thread, so this blocks that thread
async fn wait_until(deadline: Instant) {
    if let Some(sleep_for) = deadline
        .checked_duration_since(Instant::now())
        .and_then(|remaining| remaining.checked_sub(SPIN_MARGIN))
    {
        std::thread::sleep(sleep_for);
    }
    while Instant::now() < deadline {
        core::hint::spin_loop();
    }
}
