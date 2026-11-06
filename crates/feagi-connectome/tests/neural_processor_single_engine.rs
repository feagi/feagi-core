#![cfg(feature = "std")]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::{FeagiWorkerBuilder, WorkerStartError};
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::target_frequency::TargetFrequency;
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::{FeagiChannelReceiver, FeagiChannelTransmitter};
use feagi_connectome::neural_processors::singular::{
    NeuronProcessorSingleEngineRequest, NeuronProcessorSingleEngineResponse, NeuronProcessorSingleEngineVisualizationReceiver,
    NeuronProcessorSingleEngineVisualizationTransmitter, NeuronProcessorSingleEngineWorkerBuilder, NeuronProcessorSingleEngineWorkerBuilderError,
};
use feagi_connectome::connectome_basis::connectome_editing::{ConnectomeDispatch, ConnectomeEditResultantEnum, ConnectomeEditStepEnum};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantizationStandard;
use feagi_connectome_neural::neural_basis::engine::{BurstEngine, BurstEngineIndex};
use feagi_connectome_neural::neural_basis::engine::engine_change_command::EngineChangeCommandOtherError;
use feagi_connectome_neural::neural_basis::visualization::VisualizationStructureChangeCounter;

type FIQ = FeagiIndexQuantizationStandard;

/// How many bursts ran, and how many of those sent visualization
#[derive(Default)]
struct Counts {
    bursts: AtomicUsize,
    visualizations: AtomicUsize,
}

/// A burst engine that only counts its bursts, publishing how many it ran as its visualization
struct CountingEngine {
    counts: Arc<Counts>,
    /// Bursts run, never reset unlike `counts`
    bursts_run: u16,
    visualization_transmitter: NeuronProcessorSingleEngineVisualizationTransmitter<FIQ>,
}

#[derive(Debug, thiserror::Error)]
#[error("Refused to init")]
struct RefusedInit;

impl BurstEngine<FIQ, ConnectomeDispatch, ()> for CountingEngine {
    /// The burst counter, or `None` to fail init
    type InitParameters = Option<Arc<Counts>>;
    type InitError = RefusedInit;
    type VisualizationTransmitter = NeuronProcessorSingleEngineVisualizationTransmitter<FIQ>;

    async fn init_burst_engine(
        parameters: Self::InitParameters,
        _engine_index: BurstEngineIndex,
        visualization_transmitter: Self::VisualizationTransmitter,
        _burst_rate: TargetFrequency,
        _visualization_rate: TargetFrequency,
    ) -> Result<Self, Self::InitError> {
        let counts = parameters.ok_or(RefusedInit)?;
        Ok(CountingEngine { counts, bursts_run: 0, visualization_transmitter })
    }

    async fn run_burst(&mut self, send_visualization: bool) {
        self.counts.bursts.fetch_add(1, Ordering::SeqCst);
        self.bursts_run += 1;
        if send_visualization {
            self.counts.visualizations.fetch_add(1, Ordering::SeqCst);
            if let Ok(mut slot) = self.visualization_transmitter.try_send_ref() {
                slot.change_counter = VisualizationStructureChangeCounter(self.bursts_run);
            }
        }
    }

    async fn set_burst_rate(&mut self, _burst_rate: TargetFrequency) -> Result<(), EngineChangeCommandOtherError> {
        Ok(())
    }

    async fn set_visualization_rate(&mut self, _visualization_rate: TargetFrequency) -> Result<(), EngineChangeCommandOtherError> {
        Ok(())
    }

    /// Refuses every connectome edit
    async fn process_connectome_edit(&mut self, _step: ConnectomeEditStepEnum<FIQ, ConnectomeDispatch>)
        -> Result<ConnectomeEditResultantEnum<FIQ>, EngineChangeCommandOtherError> {
        Err(EngineChangeCommandOtherError::UnknownError)
    }
}

/// What a worker did while it ran
struct Run {
    counts: Arc<Counts>,
    /// Visualizations a listener received, if there was one
    received: Option<usize>,
}

/// Reads every visualization until the channel closes, checking each is newer than the last
fn listen(receiver: NeuronProcessorSingleEngineVisualizationReceiver<FIQ>) -> std::thread::JoinHandle<usize> {
    std::thread::spawn(move || block_on_thread(async move {
        let mut received = 0;
        let mut last = 0;
        while let Ok(visualization) = receiver.async_recv_ref().await {
            assert!(visualization.change_counter.0 > last, "visualizations arrived out of order");
            last = visualization.change_counter.0;
            received += 1;
        }
        received
    }))
}

/// Runs a worker for `run_for`, changing both frequencies first, and gives back what it did. With
/// `with_listener`, a listener reads its visualizations, otherwise nothing does
fn run_worker(target_frequency: f64, visualization_frequency: f64, run_for: std::time::Duration, with_listener: bool) -> Run {
    let counts = Arc::new(Counts::default());
    let builder = NeuronProcessorSingleEngineWorkerBuilder::<FIQ, CountingEngine> {
        initial_target_frequency: TargetFrequency::new_from_frequency(1000.0),
        initial_visualization_frequency: TargetFrequency::new_from_frequency(1000.0),
        engine_init_parameters: Some(counts.clone()),
    };
    let (visualization_transmitter, visualization_receiver) = builder.spawn_related_channel_pairs();
    // Kept alive without a listener, so the channel fills rather than closes
    let (listener, _unread_receiver) = match with_listener {
        true => (Some(listen(visualization_receiver)), None),
        false => (None, Some(visualization_receiver)),
    };
    let (mut worker, _command_requester) = builder.build_worker(&FeagiStdThreadSpawner, visualization_transmitter).unwrap();

    block_on_thread(async {
        let response = worker
            .make_request(NeuronProcessorSingleEngineRequest::BurstRate(TargetFrequency::new_from_frequency(target_frequency)))
            .await
            .unwrap();
        assert!(matches!(response, NeuronProcessorSingleEngineResponse::BurstRate(Ok(()))));
        let response = worker
            .make_request(NeuronProcessorSingleEngineRequest::VisualizationRate(TargetFrequency::new_from_frequency(visualization_frequency)))
            .await
            .unwrap();
        assert!(matches!(response, NeuronProcessorSingleEngineResponse::VisualizationRate(Ok(()))));
        // Only count what runs at the requested frequencies
        counts.bursts.store(0, Ordering::SeqCst);
        counts.visualizations.store(0, Ordering::SeqCst);
        std::thread::sleep(run_for);
        worker.stop().await.unwrap();
    });
    // The engine, and so the visualization transmitter, is dropped once the worker exits. That
    // closes the channel, ending the listener
    worker.join().unwrap();
    Run { counts, received: listener.map(|listener| listener.join().unwrap()) }
}

#[test]
fn bursts_then_stops() {
    let counts = run_worker(500.0, 1000.0, std::time::Duration::from_millis(50), false).counts;
    // Roughly 25 bursts at 500 Hz over 50 ms, bounded loosely for slow machines
    let bursts = counts.bursts.load(Ordering::SeqCst);
    assert!((2..=40).contains(&bursts), "ran {bursts} bursts");
}

#[test]
fn visualization_runs_at_its_own_frequency() {
    let counts = run_worker(500.0, 100.0, std::time::Duration::from_millis(200), false).counts;
    let bursts = counts.bursts.load(Ordering::SeqCst);
    let visualizations = counts.visualizations.load(Ordering::SeqCst);
    // Roughly 100 bursts and 20 visualizations, bounded loosely for slow machines
    assert!(visualizations >= 5 && visualizations * 3 < bursts, "{visualizations} visualizations over {bursts} bursts");
}

#[test]
fn visualization_faster_than_bursts_sends_every_burst() {
    let counts = run_worker(100.0, 1000.0, std::time::Duration::from_millis(100), false).counts;
    let bursts = counts.bursts.load(Ordering::SeqCst);
    assert!(bursts >= 2, "ran {bursts} bursts");
    assert_eq!(counts.visualizations.load(Ordering::SeqCst), bursts);
}

#[test]
fn listener_receives_visualizations() {
    let run = run_worker(200.0, 100.0, std::time::Duration::from_millis(100), true);
    let visualizations = run.counts.visualizations.load(Ordering::SeqCst);
    let received = run.received.unwrap();
    // A visualization is only skipped if both slots are taken, so a listener keeping up gets most.
    // Counts were reset after the listener may have received a few early ones
    assert!(received >= 2 && received <= visualizations + 4, "received {received} of {visualizations} visualizations");
}

#[test]
fn engine_init_failure_is_returned() {
    let builder = NeuronProcessorSingleEngineWorkerBuilder::<FIQ, CountingEngine> {
        initial_target_frequency: TargetFrequency::new_from_frequency(1000.0),
        initial_visualization_frequency: TargetFrequency::new_from_frequency(1000.0),
        engine_init_parameters: None,
    };
    let (visualization_transmitter, _visualization_receiver) = builder.spawn_related_channel_pairs();
    match builder.build_worker(&FeagiStdThreadSpawner, visualization_transmitter) {
        Err(WorkerStartError::WorkerError(NeuronProcessorSingleEngineWorkerBuilderError::EngineInit(RefusedInit))) => {}
        Err(other) => panic!("unexpected error: {other}"),
        Ok(_) => panic!("worker started despite engine init failing"),
    }
}

#[test]
fn edit_batch_stops_at_first_failure() {
    let builder = NeuronProcessorSingleEngineWorkerBuilder::<FIQ, CountingEngine> {
        initial_target_frequency: TargetFrequency::new_from_frequency(1000.0),
        initial_visualization_frequency: TargetFrequency::new_from_frequency(1000.0),
        engine_init_parameters: Some(Arc::new(Counts::default())),
    };
    let (visualization_transmitter, _visualization_receiver) = builder.spawn_related_channel_pairs();
    let (mut worker, _command_requester) = builder.build_worker(&FeagiStdThreadSpawner, visualization_transmitter).unwrap();

    block_on_thread(async {
        // The counting engine refuses every edit, so only the first step is attempted
        let steps = vec![ConnectomeEditStepEnum::Nothing, ConnectomeEditStepEnum::Nothing];
        let response = worker.make_request(NeuronProcessorSingleEngineRequest::ConnectomeEdits(steps)).await.unwrap();
        match response {
            NeuronProcessorSingleEngineResponse::ConnectomeEdits(results) => assert!(
                matches!(results.as_slice(), [Err(EngineChangeCommandOtherError::UnknownError)]),
                "steps after a failure should not be attempted"
            ),
            _ => panic!("edits answered with the wrong response"),
        }
        worker.stop().await.unwrap();
    });
    worker.join().unwrap();
}

#[test]
fn paused_worker_does_not_burst_but_handles_commands() {
    let counts = Arc::new(Counts::default());
    let builder = NeuronProcessorSingleEngineWorkerBuilder::<FIQ, CountingEngine> {
        initial_target_frequency: TargetFrequency::new_from_frequency(500.0),
        initial_visualization_frequency: TargetFrequency::new_from_frequency(500.0),
        engine_init_parameters: Some(counts.clone()),
    };
    let (visualization_transmitter, _visualization_receiver) = builder.spawn_related_channel_pairs();
    let (mut worker, _command_requester) = builder.build_worker(&FeagiStdThreadSpawner, visualization_transmitter).unwrap();

    block_on_thread(async {
        let response = worker.make_request(NeuronProcessorSingleEngineRequest::Pause).await.unwrap();
        assert!(matches!(response, NeuronProcessorSingleEngineResponse::Paused));
        // A burst may already have been underway when the pause arrived
        std::thread::sleep(std::time::Duration::from_millis(10));
        counts.bursts.store(0, Ordering::SeqCst);

        // Commands are still handled while paused
        let steps = vec![ConnectomeEditStepEnum::Nothing];
        let response = worker.make_request(NeuronProcessorSingleEngineRequest::ConnectomeEdits(steps)).await.unwrap();
        assert!(matches!(response, NeuronProcessorSingleEngineResponse::ConnectomeEdits(_)));
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(counts.bursts.load(Ordering::SeqCst), 0, "bursts ran while paused");

        let response = worker.make_request(NeuronProcessorSingleEngineRequest::Resume).await.unwrap();
        assert!(matches!(response, NeuronProcessorSingleEngineResponse::Resumed));
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(counts.bursts.load(Ordering::SeqCst) >= 2, "bursting did not resume");

        // Stop is still handled while paused
        worker.make_request(NeuronProcessorSingleEngineRequest::Pause).await.unwrap();
        worker.stop().await.unwrap();
    });
    worker.join().unwrap();
}

#[test]
fn command_requester_clone_reaches_the_worker() {
    let builder = NeuronProcessorSingleEngineWorkerBuilder::<FIQ, CountingEngine> {
        initial_target_frequency: TargetFrequency::new_from_frequency(1000.0),
        initial_visualization_frequency: TargetFrequency::new_from_frequency(1000.0),
        engine_init_parameters: Some(Arc::new(Counts::default())),
    };
    let (visualization_transmitter, _visualization_receiver) = builder.spawn_related_channel_pairs();
    let (mut worker, mut command_requester) = builder.build_worker(&FeagiStdThreadSpawner, visualization_transmitter).unwrap();

    block_on_thread(async {
        let response = command_requester
            .make_request(FeagiWorkerRequest::Custom(NeuronProcessorSingleEngineRequest::Pause))
            .await
            .unwrap();
        assert!(matches!(response, FeagiWorkerResponse::Custom(NeuronProcessorSingleEngineResponse::Paused)));
        worker.stop().await.unwrap();
    });
    worker.join().unwrap();
}
