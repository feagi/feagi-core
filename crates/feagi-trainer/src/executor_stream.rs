//! Sample-by-sample stream rollout (dual Misc IPU).
//!
//! Train streams Misc A + teacher Misc B and never collects the OPU. Infer drives A only
//! and scores the Misc OPU at each hold end when a motor frame arrives; a silent OPU is a
//! warning, not a failed run. The trainer does not inject Pain/Pleasure.

use std::collections::BTreeMap;

use crate::binding::decoder::SlotDecoder;
use crate::binding::encoder::TickEncoder;
use crate::binding::profile::{DecoderBindingProfile, EncoderBindingProfile, StreamMode};
use crate::binding::{FeagiRuntime, RewardPolicy};
use crate::contracts::ir_sample::{HoldEnd, Payload};
use crate::contracts::prediction_record::SCHEMA_VERSION as PREDICTION_RECORD_SCHEMA_VERSION;
use crate::contracts::TypedPrediction;
use crate::contracts::{
    IRSample, MetricScope, PredictionRecord, RunEvent, RunEventKind, RunId, TypedTarget,
};
use crate::control::{CancelToken, RunEventSink};
use crate::error::TrainerError;
use crate::executor::{
    emit_no_detection_warning, evaluate_or_empty, rollout_summary, ExecutorConfig, RolloutOutcome,
};
use crate::plugins::MetricPackPlugin;
use crate::resume::{PredictionState, ResumeFrom};
use crate::run_data::{ColumnSpec, EventTime, RunDataSink, RunDataValue, TableSpec};

/// Closed-loop stream rollout: one burst per analog sample, score at each hold end.
#[allow(clippy::too_many_arguments)]
pub fn run_stream_rollout_with_events<R, E, D, RP, M>(
    run_id: &RunId,
    samples: &[IRSample],
    runtime: &mut R,
    encoder: &mut E,
    encoder_profile: &EncoderBindingProfile,
    decoder: &mut D,
    decoder_profile: &DecoderBindingProfile,
    reward_policy: &RP,
    metric_pack: &M,
    config: &ExecutorConfig,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
    data: Option<&mut dyn RunDataSink>,
) -> Result<RolloutOutcome, TrainerError>
where
    R: FeagiRuntime,
    E: TickEncoder<Frame = R::SensoryFrame>,
    D: SlotDecoder<Frame = R::MotorFrame>,
    RP: RewardPolicy,
    M: MetricPackPlugin,
{
    run_stream_rollout_with_events_from(
        run_id,
        samples,
        runtime,
        encoder,
        encoder_profile,
        decoder,
        decoder_profile,
        reward_policy,
        metric_pack,
        config,
        events,
        cancel,
        data,
        &ResumeFrom::default(),
    )
}

/// Same as [`run_stream_rollout_with_events`], continuing from `resume` ("pause and save, resume
/// later").
///
/// Train mode resumes at sample `first_index` (a whole number of batches); infer mode at hold
/// `first_index` (the ticks after the previous hold end are replayed). With
/// `resume.boundary_stop` a skip is honoured only at the batch / hold boundary, so the segment
/// never ends in the middle of one. Infer carries the earlier holds' predictions in the engine
/// state, so a split run scores like an unsplit one.
#[allow(clippy::too_many_arguments)]
pub fn run_stream_rollout_with_events_from<R, E, D, RP, M>(
    run_id: &RunId,
    samples: &[IRSample],
    runtime: &mut R,
    encoder: &mut E,
    encoder_profile: &EncoderBindingProfile,
    decoder: &mut D,
    decoder_profile: &DecoderBindingProfile,
    reward_policy: &RP,
    metric_pack: &M,
    config: &ExecutorConfig,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
    data: Option<&mut dyn RunDataSink>,
    resume: &ResumeFrom,
) -> Result<RolloutOutcome, TrainerError>
where
    R: FeagiRuntime,
    E: TickEncoder<Frame = R::SensoryFrame>,
    D: SlotDecoder<Frame = R::MotorFrame>,
    RP: RewardPolicy,
    M: MetricPackPlugin,
{
    if config.ticks_per_sample == 0 {
        return Err(TrainerError::Config(
            "ExecutorConfig.ticks_per_sample must be non-zero".to_string(),
        ));
    }
    if config.silence_bursts > 0 {
        return Err(TrainerError::Config(
            "silence_bursts applies between discrete samples; stream presentation does not support a non-zero gap"
                .to_string(),
        ));
    }
    let stream = encoder_profile.stream.as_ref().ok_or_else(|| {
        TrainerError::Config("stream rollout requires encoder_profile.stream".to_string())
    })?;
    stream.validate()?;
    if decoder_profile.class_count == 0 {
        return Err(TrainerError::Config(
            "stream rollout requires decoder_profile.class_count > 0".to_string(),
        ));
    }

    match stream.mode {
        StreamMode::Train => run_stream_train(
            run_id,
            samples,
            runtime,
            encoder,
            encoder_profile,
            decoder,
            decoder_profile,
            reward_policy,
            metric_pack,
            config,
            events,
            cancel,
            data,
            stream.parallel_width,
            resume,
        ),
        StreamMode::Infer => run_stream_infer(
            run_id,
            samples,
            runtime,
            encoder,
            encoder_profile,
            decoder,
            decoder_profile,
            reward_policy,
            metric_pack,
            config,
            events,
            cancel,
            data,
            resume,
        ),
    }
}

const TICKS_TABLE: &str = "ticks";
const HOLDS_TABLE: &str = "holds";

fn ticks_spec() -> TableSpec {
    TableSpec {
        name: TICKS_TABLE.to_string(),
        description: "One row per presented tick and slot.".to_string(),
        columns: vec![
            ColumnSpec::unitless("sample_id", "Sample the tick belongs to."),
            ColumnSpec::unitless("slot", "Parallel slot of the batch."),
            ColumnSpec::unitless("tick", "Tick index within the sample."),
            ColumnSpec::unitless("amplitude", "Value presented at the tick."),
            ColumnSpec::unitless("target_class", "Class taught at the tick, when teaching."),
        ],
    }
}

fn holds_spec() -> TableSpec {
    TableSpec {
        name: HOLDS_TABLE.to_string(),
        description: "One row per scored hold end.".to_string(),
        columns: vec![
            ColumnSpec::unitless("sample_id", "Sample the hold belongs to."),
            ColumnSpec::unitless("tick", "Tick of the hold end."),
            ColumnSpec::unitless("target", "Class expected at the hold end."),
            ColumnSpec::unitless("predicted", "Decoded prediction."),
            ColumnSpec::unitless("correct", "Whether the prediction equals the target."),
            ColumnSpec::unitless("scores", "Per-class scores, separated by semicolons."),
        ],
    }
}

fn begin_stream_data(sink: Option<&mut (dyn RunDataSink + '_)>, mode: &str, holds: bool) {
    if let Some(sink) = sink {
        sink.declare_table(&ticks_spec());
        if holds {
            sink.declare_table(&holds_spec());
        }
        sink.event(EventTime::default(), "phase_start", mode);
    }
}

fn end_stream_data(sink: Option<&mut (dyn RunDataSink + '_)>, mode: &str, done: u64) {
    if let Some(sink) = sink {
        sink.event(
            EventTime {
                step: Some(done),
                ..EventTime::default()
            },
            "phase_end",
            mode,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn run_stream_train<R, E, D, RP, M>(
    run_id: &RunId,
    samples: &[IRSample],
    runtime: &mut R,
    encoder: &mut E,
    encoder_profile: &EncoderBindingProfile,
    _decoder: &mut D,
    decoder_profile: &DecoderBindingProfile,
    _reward_policy: &RP,
    metric_pack: &M,
    config: &ExecutorConfig,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
    mut data: Option<&mut dyn RunDataSink>,
    parallel_width: u32,
    resume: &ResumeFrom,
) -> Result<RolloutOutcome, TrainerError>
where
    R: FeagiRuntime,
    E: TickEncoder<Frame = R::SensoryFrame>,
    D: SlotDecoder<Frame = R::MotorFrame>,
    RP: RewardPolicy,
    M: MetricPackPlugin,
{
    let total_samples = samples.len() as u64;
    resume.check_within(total_samples, "sample")?;
    if !resume.first_index.is_multiple_of(u64::from(parallel_width)) {
        return Err(TrainerError::Config(format!(
            "resume index {} is not a whole number of batches of {parallel_width}",
            resume.first_index
        )));
    }
    let mut done = resume.first_index;
    begin_stream_data(data.as_deref_mut(), "train", false);

    'train: for batch in samples
        .chunks(parallel_width as usize)
        .skip((resume.first_index / u64::from(parallel_width)) as usize)
    {
        cancel.interrupt(format!("stopped after {done} of {total_samples} samples"))?;
        if cancel.is_skipped() {
            break 'train;
        }
        let series: Vec<&[f64]> = batch
            .iter()
            .map(time_series_samples)
            .collect::<Result<Vec<_>, _>>()?;
        let length = series[0].len();
        if length == 0 {
            return Err(TrainerError::Config(
                "stream train sample has an empty time series".to_string(),
            ));
        }
        for (index, values) in series.iter().enumerate() {
            if values.len() != length {
                return Err(TrainerError::Config(format!(
                    "stream train batch lengths differ (slot 0 has {length}, slot {index} has {})",
                    values.len()
                )));
            }
        }
        let class_ids: Vec<Option<u32>> = batch
            .iter()
            .map(|sample| class_id(sample).map(Some))
            .collect::<Result<Vec<_>, _>>()?;

        for tick in 0..length {
            cancel.interrupt(format!("stopped after {done} of {total_samples} samples"))?;
            if cancel.is_skipped() && !resume.boundary_stop {
                break 'train;
            }
            let amplitudes: Vec<f64> = series.iter().map(|values| values[tick]).collect();
            let frame = encoder.encode_tick(
                &amplitudes,
                &class_ids,
                encoder_profile,
                decoder_profile.class_count,
            )?;
            runtime.submit_sensory(frame)?;
            runtime.step(config.ticks_per_sample)?;
            if let Some(sink) = data.as_deref_mut() {
                for (slot, amplitude) in amplitudes.iter().enumerate() {
                    sink.row(
                        TICKS_TABLE,
                        &[
                            RunDataValue::Text(batch[slot].sample_id.0.clone()),
                            RunDataValue::count(slot as u64),
                            RunDataValue::count(tick as u64),
                            RunDataValue::Float(*amplitude),
                            class_ids[slot]
                                .map_or(RunDataValue::Null, |c| RunDataValue::Int(i64::from(c))),
                        ],
                    );
                }
            }
            events.emit(RunEvent::new(
                run_id.clone(),
                RunEventKind::Progress {
                    samples_done: done,
                    samples_total: total_samples,
                    repeat_index: 0,
                    repeat_total: 1,
                    sent_values: amplitudes,
                    tick_index: Some(tick as u64),
                    window_values: if tick == 0 {
                        series[0].to_vec()
                    } else {
                        Vec::new()
                    },
                    image_png_base64: None,
                    mask_png_base64: None,
                    preview_name: None,
                    readout: None,
                    class_scores: Vec::new(),
                    stand: None,
                },
            ));
        }

        done += batch.len() as u64;
        events.emit(RunEvent::new(
            run_id.clone(),
            RunEventKind::Progress {
                samples_done: done,
                samples_total: total_samples,
                repeat_index: 0,
                repeat_total: 1,
                sent_values: Vec::new(),
                tick_index: None,
                window_values: Vec::new(),
                image_png_base64: None,
                mask_png_base64: None,
                preview_name: None,
                readout: None,
                class_scores: Vec::new(),
                stand: None,
            },
        ));
    }

    end_stream_data(data, "train", done);
    finish_rollout(
        run_id,
        total_samples,
        done,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        metric_pack,
        events,
        cancel.is_skipped() && done < total_samples,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_stream_infer<R, E, D, RP, M>(
    run_id: &RunId,
    samples: &[IRSample],
    runtime: &mut R,
    encoder: &mut E,
    encoder_profile: &EncoderBindingProfile,
    decoder: &mut D,
    decoder_profile: &DecoderBindingProfile,
    _reward_policy: &RP,
    metric_pack: &M,
    config: &ExecutorConfig,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
    mut data: Option<&mut dyn RunDataSink>,
    resume: &ResumeFrom,
) -> Result<RolloutOutcome, TrainerError>
where
    R: FeagiRuntime,
    E: TickEncoder<Frame = R::SensoryFrame>,
    D: SlotDecoder<Frame = R::MotorFrame>,
    RP: RewardPolicy,
    M: MetricPackPlugin,
{
    let mut predictions = PredictionState::restore(resume)?;
    let (mut scored_predictions, mut scored_targets) = PredictionState::scored(&predictions);
    let total_holds: u64 = samples
        .iter()
        .map(|sample| infer_hold_ends(sample).map(|ends| ends.len() as u64))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum();
    if total_holds == 0 {
        return Err(TrainerError::Config(
            "stream infer produced no hold ends".to_string(),
        ));
    }
    resume.check_within(total_holds, "hold")?;
    let mut done = resume.first_index;
    // Resume point: the sample holding hold number `first_index`, starting at the tick after the
    // previous hold end of that sample (tick 0 when it is the sample's first hold).
    let (mut start_sample, mut start_tick) = (0usize, 0usize);
    if resume.first_index > 0 {
        let mut seen = 0u64;
        'locate: for (sample_index, sample) in samples.iter().enumerate() {
            let holds = infer_hold_ends(sample)?;
            for hold_index in 0..holds.len() {
                if seen == resume.first_index {
                    start_sample = sample_index;
                    start_tick = if hold_index == 0 {
                        0
                    } else {
                        holds[hold_index - 1].sample_index as usize + 1
                    };
                    break 'locate;
                }
                seen += 1;
            }
        }
    }
    begin_stream_data(data.as_deref_mut(), "infer", true);

    'infer: for (sample_index, sample) in samples.iter().enumerate().skip(start_sample) {
        let series = time_series_samples(sample)?;
        let holds = infer_hold_ends(sample)?;
        let hold_by_tick: BTreeMap<usize, u32> = holds
            .iter()
            .map(|hold| (hold.sample_index as usize, hold.class_id))
            .collect();
        if resume.boundary_stop && cancel.is_skipped() && done > resume.first_index {
            break 'infer;
        }
        let first_tick = if sample_index == start_sample {
            start_tick
        } else {
            0
        };
        for (tick, amplitude) in series.iter().enumerate().skip(first_tick) {
            cancel.interrupt(format!("stopped after {done} of {total_holds} holds"))?;
            if cancel.is_skipped() && !resume.boundary_stop {
                break 'infer;
            }
            let frame = encoder.encode_tick(
                &[*amplitude],
                &[None],
                encoder_profile,
                decoder_profile.class_count,
            )?;
            runtime.submit_sensory(frame)?;
            runtime.step(config.ticks_per_sample)?;
            if let Some(sink) = data.as_deref_mut() {
                sink.row(
                    TICKS_TABLE,
                    &[
                        RunDataValue::Text(sample.sample_id.0.clone()),
                        RunDataValue::Int(0),
                        RunDataValue::count(tick as u64),
                        RunDataValue::Float(*amplitude),
                        RunDataValue::Null,
                    ],
                );
            }
            events.emit(RunEvent::new(
                run_id.clone(),
                RunEventKind::Progress {
                    samples_done: done,
                    samples_total: total_holds,
                    repeat_index: 0,
                    repeat_total: 1,
                    sent_values: vec![*amplitude],
                    tick_index: Some(tick as u64),
                    window_values: Vec::new(),
                    image_png_base64: None,
                    mask_png_base64: None,
                    preview_name: None,
                    readout: None,
                    class_scores: Vec::new(),
                    stand: None,
                },
            ));
            let Some(&class_id) = hold_by_tick.get(&tick) else {
                continue;
            };
            match runtime.collect_motor()? {
                Some(motor) => {
                    let mut slots = decoder.decode_slots(motor, decoder_profile, 1)?;
                    let prediction = slots.pop().ok_or_else(|| {
                        TrainerError::Evaluation(
                            "stream infer decoder returned no slot".to_string(),
                        )
                    })?;
                    let target = TypedTarget::Class {
                        class_id,
                        label: None,
                    };
                    if let Some(sink) = data.as_deref_mut() {
                        sink.row(
                            HOLDS_TABLE,
                            &[
                                RunDataValue::Text(sample.sample_id.0.clone()),
                                RunDataValue::count(tick as u64),
                                RunDataValue::Int(i64::from(class_id)),
                                RunDataValue::Text(crate::executor::describe_prediction(
                                    &prediction,
                                )),
                                crate::executor::prediction_matches(&prediction, &target)
                                    .map_or(RunDataValue::Null, RunDataValue::Bool),
                                match &prediction {
                                    TypedPrediction::Class { scores, .. } if !scores.is_empty() => {
                                        RunDataValue::Text(
                                            scores
                                                .iter()
                                                .map(f64::to_string)
                                                .collect::<Vec<_>>()
                                                .join(";"),
                                        )
                                    }
                                    _ => RunDataValue::Null,
                                },
                            ],
                        );
                    }
                    scored_predictions.push(prediction.clone());
                    scored_targets.push(target.clone());
                    predictions.push(PredictionRecord {
                        schema_version: PREDICTION_RECORD_SCHEMA_VERSION,
                        run_id: run_id.clone(),
                        sample_id: sample.sample_id.clone(),
                        output_type: sample.output_type,
                        prediction,
                        target: Some(target),
                        timestamp: None,
                        metadata: BTreeMap::new(),
                    });
                }
                None => {
                    emit_no_detection_warning(events, run_id, &sample.sample_id);
                }
            }
            done += 1;
            events.emit(RunEvent::new(
                run_id.clone(),
                RunEventKind::Progress {
                    samples_done: done,
                    samples_total: total_holds,
                    repeat_index: 0,
                    repeat_total: 1,
                    sent_values: vec![*amplitude],
                    tick_index: Some(tick as u64),
                    window_values: Vec::new(),
                    image_png_base64: None,
                    mask_png_base64: None,
                    preview_name: None,
                    readout: None,
                    class_scores: Vec::new(),
                    stand: None,
                },
            ));
            // Pause-save stops right after a hold end that is not the sample's last (the hold
            // boundary); after the last one it waits for the sample's trailing ticks and stops
            // at the top of the next sample, so no tick is lost on resume.
            let last_of_sample = holds
                .iter()
                .all(|hold| (hold.sample_index as usize) <= tick);
            if resume.boundary_stop && cancel.is_skipped() && done < total_holds && !last_of_sample
            {
                break 'infer;
            }
        }
    }

    end_stream_data(data, "infer", done);
    finish_rollout(
        run_id,
        total_holds,
        done,
        predictions,
        scored_predictions,
        scored_targets,
        metric_pack,
        events,
        cancel.is_skipped() && done < total_holds,
    )
}

#[allow(clippy::too_many_arguments)]
fn finish_rollout<M: MetricPackPlugin>(
    run_id: &RunId,
    total_samples: u64,
    processed_samples: u64,
    predictions: Vec<PredictionRecord>,
    scored_predictions: Vec<crate::contracts::TypedPrediction>,
    scored_targets: Vec<TypedTarget>,
    metric_pack: &M,
    events: &mut dyn RunEventSink,
    skipped: bool,
) -> Result<RolloutOutcome, TrainerError> {
    let metric_result = evaluate_or_empty(metric_pack, &scored_predictions, &scored_targets)?;
    events.emit(RunEvent::new(
        run_id.clone(),
        RunEventKind::MetricUpdate {
            scope: MetricScope::Aggregate,
            class_breakdown: crate::executor::class_breakdown_event(
                &metric_result,
                &scored_targets,
            ),
            confusion: crate::executor::confusion_event(&metric_result, &scored_targets),
            metrics: metric_result.metrics.clone(),
        },
    ));
    let summary = rollout_summary(
        run_id,
        total_samples,
        processed_samples,
        scored_predictions.len() as u64,
        metric_result.metrics.clone(),
        skipped,
    );
    let resume =
        crate::resume::PredictionState::export(processed_samples, total_samples, &predictions);
    Ok(RolloutOutcome {
        summary,
        predictions,
        metric_result,
        resume,
    })
}

fn time_series_samples(sample: &IRSample) -> Result<&[f64], TrainerError> {
    match &sample.payload {
        Payload::TimeSeries {
            samples,
            channel_ids,
            ..
        } => {
            if channel_ids.len() != 1 {
                return Err(TrainerError::Config(
                    "stream presentation requires a single-channel time-series payload".to_string(),
                ));
            }
            Ok(samples.as_slice())
        }
        other => Err(TrainerError::Config(format!(
            "stream rollout requires a time-series payload, got {other:?}"
        ))),
    }
}

fn class_id(sample: &IRSample) -> Result<u32, TrainerError> {
    match &sample.target {
        Some(TypedTarget::Class { class_id, .. }) => Ok(*class_id),
        Some(other) => Err(TrainerError::Config(format!(
            "stream train requires a class target, got {other:?}"
        ))),
        None => Err(TrainerError::Config(
            "stream train requires a labeled class target".to_string(),
        )),
    }
}

fn infer_hold_ends(sample: &IRSample) -> Result<&[HoldEnd], TrainerError> {
    match &sample.payload {
        Payload::TimeSeries {
            hold_ends: Some(ends),
            ..
        } if !ends.is_empty() => Ok(ends.as_slice()),
        Payload::TimeSeries {
            hold_ends: Some(_), ..
        } => Err(TrainerError::Config(
            "stream infer sample has an empty hold_ends list".to_string(),
        )),
        Payload::TimeSeries {
            hold_ends: None, ..
        } => Err(TrainerError::Config(
            "stream infer sample is missing hold_ends".to_string(),
        )),
        other => Err(TrainerError::Config(format!(
            "stream infer requires a time-series payload, got {other:?}"
        ))),
    }
}

/// Run controls this engine's loop honours (pause, skip and cancel; at the sample or hold).
pub const CONTROL: crate::control::EngineControl = crate::control::EngineControl {
    pause: true,
    skip: true,
    cancel: true,
    boundary: crate::control::ControlBoundary::Sample,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::encoding_scheme::EncodingScheme;
    use crate::binding::profile::StreamBinding;
    use crate::binding::reward::PainPleasureReward;
    use crate::binding::StubFeagiRuntime;
    use crate::contracts::common::{
        DatasetVersionId, Modality, OutputType, PluginId, SampleId, Split,
    };
    use crate::contracts::prediction_record::TypedPrediction;
    use crate::contracts::PluginRef;
    use crate::control::NoopEventSink;
    use crate::metrics::ClassificationMetricPack;
    use std::collections::BTreeMap;

    struct EchoTickEncoder;

    impl TickEncoder for EchoTickEncoder {
        type Frame = Vec<f64>;

        fn plugin_ref(&self) -> PluginRef {
            PluginRef {
                id: PluginId("test.echo_tick".to_string()),
                version: "1.0.0".to_string(),
            }
        }

        fn encode_tick(
            &mut self,
            amplitudes: &[f64],
            class_ids: &[Option<u32>],
            _profile: &EncoderBindingProfile,
            class_count: u32,
        ) -> Result<Self::Frame, TrainerError> {
            let mut out = amplitudes.to_vec();
            let mut hot = vec![0.0; class_count as usize];
            if let Some(class) = class_ids.first().copied().flatten() {
                hot[class as usize] = 1.0;
            }
            out.extend(hot);
            Ok(out)
        }
    }

    struct TailArgmax;

    impl SlotDecoder for TailArgmax {
        type Frame = Vec<f64>;

        fn plugin_ref(&self) -> PluginRef {
            PluginRef {
                id: PluginId("test.tail_argmax".to_string()),
                version: "1.0.0".to_string(),
            }
        }

        fn decode_slots(
            &mut self,
            motor: Self::Frame,
            profile: &DecoderBindingProfile,
            slot_count: u32,
        ) -> Result<Vec<TypedPrediction>, TrainerError> {
            if slot_count != 1 {
                return Err(TrainerError::Config(
                    "test tail argmax supports slot_count = 1".to_string(),
                ));
            }
            let start = motor.len().saturating_sub(profile.class_count as usize);
            let scores = motor[start..].to_vec();
            let readout = crate::binding::class_from_scores(&scores);
            Ok(vec![TypedPrediction::Class { readout, scores }])
        }
    }

    fn stream_profile(mode: StreamMode) -> EncoderBindingProfile {
        EncoderBindingProfile {
            cortical_area_id: "misc_amp".to_string(),
            channels: 1,
            scheme: EncodingScheme::Value,
            image_width: None,
            image_height: None,
            vision_layout: crate::binding::profile::VisionLayout::Simple,
            cortical_name: None,
            stream: Some(StreamBinding {
                parallel_width: 1,
                amplitude_unit: 0,
                teacher_unit: 1,
                teacher_cortical_area_id: "misc_class".to_string(),
                teacher_cortical_name: None,
                mode,
            }),
            teacher: None,
            segmentation_teacher: None,
            segmented_vision: None,
        }
    }

    fn decoder_profile() -> DecoderBindingProfile {
        DecoderBindingProfile {
            cortical_area_id: "misc_opu".to_string(),
            class_count: 3,
            bins: 1,
            mask_width: None,
            mask_height: None,
            cortical_name: None,
        }
    }

    fn beat(class_id: u32, values: Vec<f64>) -> IRSample {
        IRSample {
            schema_version: crate::contracts::ir_sample::SCHEMA_VERSION,
            sample_id: SampleId(format!("beat-{class_id}")),
            dataset_version_id: DatasetVersionId("t@1".to_string()),
            split: Split::Train,
            modality: Modality::TimeSeries,
            payload: Payload::TimeSeries {
                sample_rate_hz: 360.0,
                channel_ids: vec!["lead_0".to_string()],
                samples: values,
                hold_ends: None,
            },
            target: Some(TypedTarget::Class {
                class_id,
                label: None,
            }),
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn train_streams_ticks_without_collecting_motor() {
        let samples = vec![beat(1, vec![0.1, 0.2, 0.3])];
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = EchoTickEncoder;
        let mut decoder = TailArgmax;
        let reward = PainPleasureReward::new(0.5).unwrap();
        let metric = ClassificationMetricPack::new();
        let mut sink = NoopEventSink;
        let mut data = crate::run_data::RecordingRunDataSink::default();
        let outcome = run_stream_rollout_with_events(
            &RunId("run-stream".to_string()),
            &samples,
            &mut runtime,
            &mut encoder,
            &stream_profile(StreamMode::Train),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &ExecutorConfig {
                ticks_per_sample: 2,
                silence_bursts: 0,
            },
            &mut sink,
            &CancelToken::new(),
            Some(&mut data),
        )
        .expect("rollout");
        assert!(data.errors.is_empty(), "{:?}", data.errors);
        assert_eq!(data.rows_of("ticks").len(), 3);
        assert_eq!(data.event_kinds(), vec!["phase_start", "phase_end"]);
        assert_eq!(runtime.burst_count(), 6);
        assert_eq!(runtime.motor_collects(), 0);
        assert!(runtime.submitted_rewards().is_empty());
        assert!(outcome.predictions.is_empty());
        assert_eq!(outcome.summary.evaluated_samples, 0);
    }

    #[test]
    fn stream_rejects_silence_bursts() {
        let samples = vec![beat(1, vec![0.1])];
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = EchoTickEncoder;
        let mut decoder = TailArgmax;
        let reward = PainPleasureReward::new(0.5).unwrap();
        let metric = ClassificationMetricPack::new();
        let mut sink = NoopEventSink;
        let result = run_stream_rollout_with_events(
            &RunId("run-stream-gap".to_string()),
            &samples,
            &mut runtime,
            &mut encoder,
            &stream_profile(StreamMode::Train),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &ExecutorConfig {
                ticks_per_sample: 1,
                silence_bursts: 2,
            },
            &mut sink,
            &CancelToken::new(),
            None,
        );
        assert!(matches!(result, Err(TrainerError::Config(_))));
    }

    #[test]
    fn train_emits_window_once_and_tick_on_every_sample() {
        use crate::control::CollectingEventSink;

        let samples = vec![beat(1, vec![0.1, 0.2, 0.3])];
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = EchoTickEncoder;
        let mut decoder = TailArgmax;
        let reward = PainPleasureReward::new(0.5).unwrap();
        let metric = ClassificationMetricPack::new();
        let mut sink = CollectingEventSink::default();
        run_stream_rollout_with_events(
            &RunId("run-stream-preview".to_string()),
            &samples,
            &mut runtime,
            &mut encoder,
            &stream_profile(StreamMode::Train),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &ExecutorConfig {
                ticks_per_sample: 1,
                silence_bursts: 0,
            },
            &mut sink,
            &CancelToken::new(),
            None,
        )
        .expect("rollout");
        let ticks: Vec<(Option<u64>, Vec<f64>, Vec<f64>)> = sink
            .events
            .iter()
            .filter_map(|event| match &event.kind {
                RunEventKind::Progress {
                    tick_index,
                    window_values,
                    sent_values,
                    ..
                } if tick_index.is_some() => {
                    Some((*tick_index, window_values.clone(), sent_values.clone()))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            ticks,
            vec![
                (Some(0), vec![0.1, 0.2, 0.3], vec![0.1]),
                (Some(1), Vec::new(), vec![0.2]),
                (Some(2), Vec::new(), vec![0.3]),
            ]
        );
    }

    #[test]
    fn infer_scores_only_at_hold_end() {
        let sample = IRSample {
            schema_version: crate::contracts::ir_sample::SCHEMA_VERSION,
            sample_id: SampleId("ep".to_string()),
            dataset_version_id: DatasetVersionId("t@1".to_string()),
            split: Split::Test,
            modality: Modality::TimeSeries,
            payload: Payload::TimeSeries {
                sample_rate_hz: 360.0,
                channel_ids: vec!["lead_0".to_string()],
                samples: vec![0.1, 0.2, 0.3, 0.4],
                hold_ends: Some(vec![HoldEnd {
                    sample_index: 2,
                    class_id: 0,
                }]),
            },
            target: None,
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        };
        let mut runtime = StubFeagiRuntime::new(
            |sensory| {
                let mut motor = vec![0.0, 0.0, 0.0];
                motor[0] = 1.0;
                let _ = sensory;
                motor
            },
            false,
        );
        let mut encoder = EchoTickEncoder;
        let mut decoder = TailArgmax;
        let reward = PainPleasureReward::new(0.5).unwrap();
        let metric = ClassificationMetricPack::new();
        let mut sink = NoopEventSink;
        let mut data = crate::run_data::RecordingRunDataSink::default();
        let outcome = run_stream_rollout_with_events(
            &RunId("run-infer".to_string()),
            &[sample],
            &mut runtime,
            &mut encoder,
            &stream_profile(StreamMode::Infer),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &ExecutorConfig {
                ticks_per_sample: 1,
                silence_bursts: 0,
            },
            &mut sink,
            &CancelToken::new(),
            Some(&mut data),
        )
        .expect("rollout");
        assert!(data.errors.is_empty(), "{:?}", data.errors);
        assert_eq!(data.rows_of("ticks").len(), 4);
        assert_eq!(data.rows_of("holds").len(), 1);
        assert_eq!(data.event_kinds(), vec!["phase_start", "phase_end"]);
        assert_eq!(runtime.burst_count(), 4);
        assert_eq!(outcome.predictions.len(), 1);
        assert!(runtime.submitted_rewards().is_empty());
        assert_eq!(runtime.motor_collects(), 1);
    }

    #[test]
    fn infer_silent_motor_warns_and_completes() {
        use crate::control::CollectingEventSink;
        use crate::executor::NO_DETECTION_RESULT_WARNING;

        let sample = IRSample {
            schema_version: crate::contracts::ir_sample::SCHEMA_VERSION,
            sample_id: SampleId("ep-silent".to_string()),
            dataset_version_id: DatasetVersionId("t@1".to_string()),
            split: Split::Test,
            modality: Modality::TimeSeries,
            payload: Payload::TimeSeries {
                sample_rate_hz: 360.0,
                channel_ids: vec!["lead_0".to_string()],
                samples: vec![0.1, 0.2, 0.3],
                hold_ends: Some(vec![HoldEnd {
                    sample_index: 2,
                    class_id: 0,
                }]),
            },
            target: None,
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        };
        let mut runtime = StubFeagiRuntime::silent();
        let mut encoder = EchoTickEncoder;
        let mut decoder = TailArgmax;
        let reward = PainPleasureReward::new(0.5).unwrap();
        let metric = ClassificationMetricPack::new();
        let mut sink = CollectingEventSink::default();
        let outcome = run_stream_rollout_with_events(
            &RunId("run-infer-silent".to_string()),
            &[sample],
            &mut runtime,
            &mut encoder,
            &stream_profile(StreamMode::Infer),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &ExecutorConfig {
                ticks_per_sample: 1,
                silence_bursts: 0,
            },
            &mut sink,
            &CancelToken::new(),
            None,
        )
        .expect("rollout");
        assert_eq!(
            outcome.summary.status,
            crate::contracts::RunStatus::Completed
        );
        assert!(outcome.predictions.is_empty());
        assert_eq!(runtime.motor_collects(), 1);
        let warnings: Vec<_> = sink
            .events
            .iter()
            .filter_map(|event| match &event.kind {
                RunEventKind::Warning { message } => Some(message.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains(NO_DETECTION_RESULT_WARNING));
        assert!(warnings[0].contains("ep-silent"));
    }

    #[test]
    fn infer_skip_after_first_tick_scores_holds_already_seen() {
        use crate::control::CollectingEventSink;

        struct SkipAfterFirstStep {
            inner: StubFeagiRuntime,
            cancel: CancelToken,
            stepped: bool,
        }

        impl FeagiRuntime for SkipAfterFirstStep {
            type SensoryFrame = Vec<f64>;
            type MotorFrame = Vec<f64>;

            fn submit_sensory(&mut self, frame: Self::SensoryFrame) -> Result<(), TrainerError> {
                self.inner.submit_sensory(frame)
            }

            fn submit_reward(
                &mut self,
                signals: &[crate::binding::reward::RewardSignal],
            ) -> Result<(), TrainerError> {
                self.inner.submit_reward(signals)
            }

            fn step(&mut self, ticks: u32) -> Result<(), TrainerError> {
                self.inner.step(ticks)?;
                if !self.stepped {
                    self.stepped = true;
                    self.cancel.skip();
                }
                Ok(())
            }

            fn collect_motor(&mut self) -> Result<Option<Self::MotorFrame>, TrainerError> {
                self.inner.collect_motor()
            }
        }

        let sample = IRSample {
            schema_version: crate::contracts::ir_sample::SCHEMA_VERSION,
            sample_id: SampleId("ep-skip".to_string()),
            dataset_version_id: DatasetVersionId("t@1".to_string()),
            split: Split::Test,
            modality: Modality::TimeSeries,
            payload: Payload::TimeSeries {
                sample_rate_hz: 360.0,
                channel_ids: vec!["lead_0".to_string()],
                samples: vec![0.1, 0.2, 0.3, 0.4],
                hold_ends: Some(vec![
                    HoldEnd {
                        sample_index: 0,
                        class_id: 0,
                    },
                    HoldEnd {
                        sample_index: 2,
                        class_id: 0,
                    },
                ]),
            },
            target: None,
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        };
        let cancel = CancelToken::new();
        let mut runtime = SkipAfterFirstStep {
            inner: StubFeagiRuntime::new(
                |sensory| {
                    let mut motor = vec![0.0, 0.0, 0.0];
                    motor[0] = 1.0;
                    let _ = sensory;
                    motor
                },
                false,
            ),
            cancel: cancel.clone(),
            stepped: false,
        };
        let mut encoder = EchoTickEncoder;
        let mut decoder = TailArgmax;
        let reward = PainPleasureReward::new(0.5).unwrap();
        let metric = ClassificationMetricPack::new();
        let mut sink = CollectingEventSink::default();
        let outcome = run_stream_rollout_with_events(
            &RunId("run-infer-skip".to_string()),
            &[sample],
            &mut runtime,
            &mut encoder,
            &stream_profile(StreamMode::Infer),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &ExecutorConfig {
                ticks_per_sample: 1,
                silence_bursts: 0,
            },
            &mut sink,
            &cancel,
            None,
        )
        .expect("skip scores");
        assert_eq!(outcome.summary.status, crate::contracts::RunStatus::Skipped);
        assert_eq!(outcome.summary.total_samples, 2);
        assert_eq!(outcome.summary.processed_samples, Some(1));
        assert_eq!(outcome.summary.evaluated_samples, 1);
        assert_eq!(outcome.predictions.len(), 1);
        assert_eq!(runtime.inner.burst_count(), 1);
    }

    /// Runtime wrapper that asks for a skip (the pause-stop) after `at` steps.
    struct SkipAtStep {
        inner: StubFeagiRuntime,
        cancel: CancelToken,
        at: u32,
        steps: u32,
    }

    impl FeagiRuntime for SkipAtStep {
        type SensoryFrame = Vec<f64>;
        type MotorFrame = Vec<f64>;

        fn submit_sensory(&mut self, frame: Self::SensoryFrame) -> Result<(), TrainerError> {
            self.inner.submit_sensory(frame)
        }

        fn submit_reward(
            &mut self,
            signals: &[crate::binding::reward::RewardSignal],
        ) -> Result<(), TrainerError> {
            self.inner.submit_reward(signals)
        }

        fn step(&mut self, ticks: u32) -> Result<(), TrainerError> {
            self.inner.step(ticks)?;
            self.steps += 1;
            if self.steps == self.at {
                self.cancel.skip();
            }
            Ok(())
        }

        fn collect_motor(&mut self) -> Result<Option<Self::MotorFrame>, TrainerError> {
            self.inner.collect_motor()
        }
    }

    fn class_zero_runtime(cancel: &CancelToken, at: Option<u32>) -> SkipAtStep {
        SkipAtStep {
            inner: StubFeagiRuntime::new(
                |_| {
                    let mut motor = vec![0.0, 0.0, 0.0];
                    motor[0] = 1.0;
                    motor
                },
                false,
            ),
            cancel: cancel.clone(),
            at: at.unwrap_or(u32::MAX),
            steps: 0,
        }
    }

    fn infer_episode(id: &str, holds: &[(u64, u32)]) -> IRSample {
        IRSample {
            schema_version: crate::contracts::ir_sample::SCHEMA_VERSION,
            sample_id: SampleId(id.to_string()),
            dataset_version_id: DatasetVersionId("t@1".to_string()),
            split: Split::Test,
            modality: Modality::TimeSeries,
            payload: Payload::TimeSeries {
                sample_rate_hz: 360.0,
                channel_ids: vec!["lead_0".to_string()],
                samples: vec![0.1, 0.2, 0.3, 0.4, 0.5],
                hold_ends: Some(
                    holds
                        .iter()
                        .map(|(sample_index, class_id)| HoldEnd {
                            sample_index: *sample_index,
                            class_id: *class_id,
                        })
                        .collect(),
                ),
            },
            target: None,
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        }
    }

    fn run_stream(
        samples: &[IRSample],
        profile: &EncoderBindingProfile,
        from: &ResumeFrom,
        skip_at: Option<u32>,
    ) -> (RolloutOutcome, u64, usize) {
        let cancel = CancelToken::new();
        let mut runtime = class_zero_runtime(&cancel, skip_at);
        let mut encoder = EchoTickEncoder;
        let mut decoder = TailArgmax;
        let reward = PainPleasureReward::new(0.5).unwrap();
        let metric = ClassificationMetricPack::new();
        let mut sink = NoopEventSink;
        let mut data = crate::run_data::RecordingRunDataSink::default();
        let outcome = run_stream_rollout_with_events_from(
            &RunId("run-stream-split".to_string()),
            samples,
            &mut runtime,
            &mut encoder,
            profile,
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &ExecutorConfig {
                ticks_per_sample: 1,
                silence_bursts: 0,
            },
            &mut sink,
            &cancel,
            Some(&mut data),
            from,
        )
        .expect("rollout");
        let bursts = runtime.inner.burst_count();
        let tick_rows = data.rows_of("ticks").len();
        (outcome, bursts, tick_rows)
    }

    #[test]
    fn infer_split_at_a_hold_equals_unsplit_including_a_mid_sample_resume() {
        let samples = vec![
            infer_episode("ep-a", &[(1, 0), (3, 1)]),
            infer_episode("ep-b", &[(2, 0), (4, 1)]),
        ];
        let profile = stream_profile(StreamMode::Infer);
        let (unsplit, unsplit_bursts, unsplit_rows) =
            run_stream(&samples, &profile, &ResumeFrom::default(), None);
        assert_eq!(unsplit.summary.evaluated_samples, 4);
        // Pause asked at step 1 (the first hold's tick) and at step 3 (inside the second hold).
        for (skip_at, expected_done) in [(1u32, 1u64), (3, 2)] {
            let (first, first_bursts, first_rows) = run_stream(
                &samples,
                &profile,
                &ResumeFrom::default().at_boundary(),
                Some(skip_at),
            );
            assert!(first.resume.partial);
            assert_eq!(
                first.resume.units_done, expected_done,
                "stops at a hold end"
            );
            let export: crate::resume::ResumeExport =
                serde_json::from_str(&serde_json::to_string(&first.resume).unwrap()).unwrap();
            let (second, second_bursts, second_rows) =
                run_stream(&samples, &profile, &ResumeFrom::from_export(&export), None);
            assert!(!second.resume.partial);
            assert_eq!(second.predictions, unsplit.predictions);
            assert_eq!(second.metric_result, unsplit.metric_result);
            assert_eq!(
                second.summary.evaluated_samples,
                unsplit.summary.evaluated_samples
            );
            assert_eq!(second.summary.status, unsplit.summary.status);
            assert_eq!(
                first_bursts + second_bursts,
                unsplit_bursts,
                "no tick lost or repeated"
            );
            assert_eq!(first_rows + second_rows, unsplit_rows);
        }
    }

    #[test]
    fn train_pause_stops_at_the_batch_and_resumes_at_the_next_one() {
        let samples: Vec<IRSample> = (0..4).map(|i| beat(i % 3, vec![0.1, 0.2, 0.3])).collect();
        let mut profile = stream_profile(StreamMode::Train);
        profile.stream.as_mut().unwrap().parallel_width = 2;
        let (unsplit, unsplit_bursts, _) =
            run_stream(&samples, &profile, &ResumeFrom::default(), None);
        assert_eq!(
            unsplit.summary.status,
            crate::contracts::RunStatus::Completed
        );

        // Without the boundary mode a skip cuts the batch after its first tick.
        let (cut, _, _) = run_stream(&samples, &profile, &ResumeFrom::default(), Some(1));
        assert_eq!(cut.resume.units_done, 0);

        let (first, first_bursts, _) = run_stream(
            &samples,
            &profile,
            &ResumeFrom::default().at_boundary(),
            Some(1),
        );
        assert!(first.resume.partial);
        assert_eq!(first.resume.units_done, 2, "the batch in progress finished");
        let (second, second_bursts, _) = run_stream(
            &samples,
            &profile,
            &ResumeFrom::from_export(&first.resume),
            None,
        );
        assert!(!second.resume.partial);
        assert_eq!(
            second.summary.status,
            crate::contracts::RunStatus::Completed
        );
        assert_eq!(second.summary.total_samples, 4);
        assert_eq!(first_bursts + second_bursts, unsplit_bursts);
    }

    #[test]
    fn train_resume_off_a_batch_boundary_is_refused() {
        let samples: Vec<IRSample> = (0..4).map(|i| beat(i % 3, vec![0.1])).collect();
        let mut profile = stream_profile(StreamMode::Train);
        profile.stream.as_mut().unwrap().parallel_width = 2;
        let cancel = CancelToken::new();
        let mut runtime = class_zero_runtime(&cancel, None);
        let result = run_stream_rollout_with_events_from(
            &RunId("r".to_string()),
            &samples,
            &mut runtime,
            &mut EchoTickEncoder,
            &profile,
            &mut TailArgmax,
            &decoder_profile(),
            &PainPleasureReward::new(0.5).unwrap(),
            &ClassificationMetricPack::new(),
            &ExecutorConfig {
                ticks_per_sample: 1,
                silence_bursts: 0,
            },
            &mut NoopEventSink,
            &cancel,
            None,
            &ResumeFrom {
                first_index: 1,
                ..ResumeFrom::default()
            },
        );
        assert!(matches!(result, Err(TrainerError::Config(_))));
    }
}
