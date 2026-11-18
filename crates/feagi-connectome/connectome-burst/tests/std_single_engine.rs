//! Tests the single engine NPU worker (`NeuronProcessorSingleEngineWorkerBuilder`) with a
//! test-local engine that only counts bursts and visualization sends, so the burst loop's timing,
//! pausing and command handling can be observed from outside the worker thread.
//!
//! Run with `cargo test -p connectome-burst --features std,burst-basis/allow-reshape-edits --test std_single_engine`
//! (`ConnectomeEditEnumHandler` requires the reshape edit handler, which only builds with that
//! feature for now)

use burst_basis::connectome_editing::{
    ConnectomeEditEnumHandler, ConnectomeEditError, ConnectomeEditHandler, ConnectomeEditNothing,
    ConnectomeEditReshapeStep, ConnectomeEditStep,
};
use burst_basis::multi_engine::{BurstEngineIndex, SingleEngineIndexing};
use burst_basis::visualization::{AllocVisualizationReceiver, AllocVisualizationTransmitter, BurstEngineVisualizationFull};
use burst_basis::BurstEngine;
use connectome_burst::npu_workers::std_single_engine::NeuronProcessorSingleEngineWorkerBuilder;
use connectome_burst::worker_commands::{NeuralProcessingUnitRequest, NeuralProcessingUnitResultant};
use connectome_basis::quantization_levels::FeagiIndexQuantizationStandard;
use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend;
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::target_frequency::TargetFrequency;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

type Fiq = FeagiIndexQuantizationStandard;
type Vis = BurstEngineVisualizationFull<Fiq, AllocVisualizationTransmitter<Fiq>>;
type Request = NeuralProcessingUnitRequest<Fiq, SingleEngineIndexing>;
type Response = NeuralProcessingUnitResultant<Fiq, SingleEngineIndexing>;
type Worker = FeagiWorker<FeagiStdThreadSpawner, FeagiRequestResponseChannelsAllocBackend, Request, Response>;

//region Counting engine

/// What the counting engine has done, readable from the test thread
#[derive(Default)]
struct Counters {
    bursts: AtomicU64,
    visualizations: AtomicU64,
}

impl Counters {
    fn bursts(&self) -> u64 {
        self.bursts.load(Ordering::SeqCst)
    }

    fn visualizations(&self) -> u64 {
        self.visualizations.load(Ordering::SeqCst)
    }
}

/// Runs no neurons. Each burst only counts itself, and whether it was asked to send visualization
struct CountingEngine {
    counters: Arc<Counters>,
    _visualization_transmitter: AllocVisualizationTransmitter<Fiq>,
}

#[derive(Debug)]
struct CountingEngineInitError;

impl core::fmt::Display for CountingEngineInitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "counting engine failed to init")
    }
}

impl core::error::Error for CountingEngineInitError {}

impl BurstEngine<Fiq, Vis> for CountingEngine {
    type InitParameters = Arc<Counters>;
    type InitError = CountingEngineInitError;

    async fn init_burst_engine(
        parameters: Self::InitParameters,
        _engine_index: BurstEngineIndex,
        visualization_transmitter: AllocVisualizationTransmitter<Fiq>,
    ) -> Result<Self, Self::InitError> {
        Ok(Self {
            counters: parameters,
            _visualization_transmitter: visualization_transmitter,
        })
    }

    async fn run_burst(&mut self, send_visualization: bool) {
        self.counters.bursts.fetch_add(1, Ordering::SeqCst);
        if send_visualization {
            self.counters.visualizations.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl ConnectomeEditEnumHandler<Fiq> for CountingEngine {}

// The tests never send an edit through to the engine (only an empty batch), and the resultants
// cannot be constructed outside burst-basis, so both handlers just refuse
impl ConnectomeEditHandler<Fiq, ConnectomeEditNothing<Fiq>> for CountingEngine {
    async fn handle_edit(
        &mut self,
        _edit: ConnectomeEditNothing<Fiq>,
    ) -> Result<<ConnectomeEditNothing<Fiq> as ConnectomeEditStep<Fiq>>::Resultant, ConnectomeEditError<Fiq>> {
        Err(ConnectomeEditError::GeneralEngineFailure("counting engine does not edit", false))
    }
}

impl ConnectomeEditHandler<Fiq, ConnectomeEditReshapeStep<Fiq>> for CountingEngine {
    async fn handle_edit(
        &mut self,
        _edit: ConnectomeEditReshapeStep<Fiq>,
    ) -> Result<<ConnectomeEditReshapeStep<Fiq> as ConnectomeEditStep<Fiq>>::Resultant, ConnectomeEditError<Fiq>> {
        Err(ConnectomeEditError::GeneralEngineFailure("counting engine does not edit", false))
    }
}

//endregion

//region Helpers

/// A running worker plus what is needed to observe it. The visualization receiver is held so the
/// channel stays open for the worker's lifetime
struct Harness {
    worker: Worker,
    counters: Arc<Counters>,
    _visualization_receiver: AllocVisualizationReceiver<Fiq>,
}

fn hz(frequency: f64) -> TargetFrequency {
    TargetFrequency::new_from_frequency(frequency).expect("valid test frequency")
}

fn start(burst_hz: f64, visualization_hz: f64) -> Harness {
    let counters = Arc::new(Counters::default());
    let builder = NeuronProcessorSingleEngineWorkerBuilder::<Fiq, Vis, CountingEngine> {
        initial_target_frequency: hz(burst_hz),
        initial_visualization_frequency: hz(visualization_hz),
        engine_init_parameters: counters.clone(),
    };
    let (visualization_transmitter, visualization_receiver) = builder.spawn_related_channel_pairs();
    let (worker, ()) = builder
        .build_worker(&FeagiStdThreadSpawner, visualization_transmitter)
        .expect("worker starts");
    Harness {
        worker,
        counters,
        _visualization_receiver: visualization_receiver,
    }
}

fn request(harness: &mut Harness, request: Request) -> Response {
    block_on_thread(harness.worker.make_request(request)).expect("worker replies")
}

/// Stops the worker and checks its thread exits without panicking
fn shutdown(mut harness: Harness) {
    block_on_thread(harness.worker.stop()).expect("worker acknowledges stop");
    harness.worker.join().expect("worker thread exits cleanly");
}

/// How many bursts and visualizations happen over `window`
fn count_over(counters: &Counters, window: Duration) -> (u64, u64) {
    let (bursts, visualizations) = (counters.bursts(), counters.visualizations());
    std::thread::sleep(window);
    (counters.bursts() - bursts, counters.visualizations() - visualizations)
}

/// Lets the worker get going (or settle after a rate change) before measuring
const SETTLE: Duration = Duration::from_millis(100);
const WINDOW: Duration = Duration::from_millis(500);

//endregion

#[test]
fn bursts_and_visualizes_at_their_target_rates() {
    let harness = start(100.0, 20.0);
    std::thread::sleep(SETTLE);

    // 100 Hz over 500 ms is 50 bursts, 20 Hz is 10 visualizations. Tolerances allow for a busy
    // test machine
    let (bursts, visualizations) = count_over(&harness.counters, WINDOW);
    assert!((40..=60).contains(&bursts), "expected ~50 bursts in {WINDOW:?}, got {bursts}");
    assert!((7..=13).contains(&visualizations), "expected ~10 visualizations in {WINDOW:?}, got {visualizations}");
    assert!(harness.counters.visualizations() <= harness.counters.bursts());

    shutdown(harness);
}

#[test]
fn visualization_never_outpaces_bursts() {
    // Visualization asked for faster than bursts, so every burst visualizes
    let harness = start(50.0, 500.0);
    std::thread::sleep(SETTLE);

    let (bursts, visualizations) = count_over(&harness.counters, WINDOW);
    assert!(bursts > 0);
    assert_eq!(bursts, visualizations, "every burst should visualize, and no more");

    shutdown(harness);
}

#[test]
fn burst_rate_change_applies() {
    let mut harness = start(100.0, 1.0);
    assert!(matches!(request(&mut harness, NeuralProcessingUnitRequest::BurstRate(hz(20.0))), NeuralProcessingUnitResultant::BurstRate));
    std::thread::sleep(SETTLE);

    // 20 Hz over 500 ms is 10 bursts
    let (bursts, _) = count_over(&harness.counters, WINDOW);
    assert!((7..=13).contains(&bursts), "expected ~10 bursts in {WINDOW:?} after slowing to 20 Hz, got {bursts}");

    shutdown(harness);
}

#[test]
fn pause_stops_bursting_until_resumed() {
    let mut harness = start(100.0, 1.0);
    std::thread::sleep(SETTLE);

    // Once the pause is acknowledged, not a single further burst may run
    assert!(matches!(request(&mut harness, NeuralProcessingUnitRequest::Pause), NeuralProcessingUnitResultant::Paused));
    let (bursts, _) = count_over(&harness.counters, Duration::from_millis(200));
    assert_eq!(bursts, 0, "bursted while paused");

    // Commands are still handled while paused
    assert!(matches!(request(&mut harness, NeuralProcessingUnitRequest::BurstRate(hz(100.0))), NeuralProcessingUnitResultant::BurstRate));
    assert_eq!(count_over(&harness.counters, Duration::from_millis(50)).0, 0, "a command while paused caused a burst");

    assert!(matches!(request(&mut harness, NeuralProcessingUnitRequest::Resume), NeuralProcessingUnitResultant::Resumed));
    let (bursts, _) = count_over(&harness.counters, Duration::from_millis(200));
    assert!(bursts > 0, "did not burst after resuming");

    shutdown(harness);
}

#[test]
fn connectome_edits_pause_until_resumed() {
    let mut harness = start(100.0, 1.0);
    std::thread::sleep(SETTLE);

    // An empty batch still counts as starting an edit
    match request(&mut harness, NeuralProcessingUnitRequest::ConnectomeEdits(vec![])) {
        NeuralProcessingUnitResultant::ConnectomeEdits(Ok(results)) => assert!(results.is_empty()),
        _ => panic!("expected an empty, successful ConnectomeEdits reply"),
    }
    let (bursts, _) = count_over(&harness.counters, Duration::from_millis(200));
    assert_eq!(bursts, 0, "bursted after an edit without being resumed");

    assert!(matches!(request(&mut harness, NeuralProcessingUnitRequest::Resume), NeuralProcessingUnitResultant::Resumed));
    let (bursts, _) = count_over(&harness.counters, Duration::from_millis(200));
    assert!(bursts > 0, "did not burst after resuming from an edit");

    shutdown(harness);
}

#[test]
fn stopped_worker_refuses_requests() {
    let mut harness = start(100.0, 1.0);
    block_on_thread(harness.worker.stop()).expect("worker acknowledges stop");
    assert!(block_on_thread(harness.worker.make_request(NeuralProcessingUnitRequest::Pause)).is_err());
    harness.worker.join().expect("worker thread exits cleanly");
}
