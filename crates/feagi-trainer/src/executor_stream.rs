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
use crate::contracts::run_summary::SCHEMA_VERSION as RUN_SUMMARY_SCHEMA_VERSION;
use crate::contracts::{
    IRSample, MetricScope, PredictionRecord, RunEvent, RunEventKind, RunId, RunStatus, RunSummary,
    TypedTarget,
};
use crate::control::{CancelToken, RunEventSink};
use crate::error::TrainerError;
use crate::executor::{
    emit_no_detection_warning, evaluate_or_empty, ExecutorConfig, RolloutOutcome,
};
use crate::plugins::MetricPackPlugin;

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
            stream.parallel_width,
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
        ),
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
    parallel_width: u32,
) -> Result<RolloutOutcome, TrainerError>
where
    R: FeagiRuntime,
    E: TickEncoder<Frame = R::SensoryFrame>,
    D: SlotDecoder<Frame = R::MotorFrame>,
    RP: RewardPolicy,
    M: MetricPackPlugin,
{
    let total_samples = samples.len() as u64;
    let mut done = 0u64;

    for batch in samples.chunks(parallel_width as usize) {
        cancel.interrupt(format!("stopped after {done} of {total_samples} samples"))?;
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
            let amplitudes: Vec<f64> = series.iter().map(|values| values[tick]).collect();
            let frame = encoder.encode_tick(
                &amplitudes,
                &class_ids,
                encoder_profile,
                decoder_profile.class_count,
            )?;
            runtime.submit_sensory(frame)?;
            runtime.step(config.ticks_per_sample)?;
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
            },
        ));
    }

    finish_rollout(
        run_id,
        total_samples,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        metric_pack,
        events,
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
) -> Result<RolloutOutcome, TrainerError>
where
    R: FeagiRuntime,
    E: TickEncoder<Frame = R::SensoryFrame>,
    D: SlotDecoder<Frame = R::MotorFrame>,
    RP: RewardPolicy,
    M: MetricPackPlugin,
{
    let mut predictions = Vec::new();
    let mut scored_predictions = Vec::new();
    let mut scored_targets = Vec::new();
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
    let mut done = 0u64;

    for sample in samples {
        let series = time_series_samples(sample)?;
        let holds = infer_hold_ends(sample)?;
        let hold_by_tick: BTreeMap<usize, u32> = holds
            .iter()
            .map(|hold| (hold.sample_index as usize, hold.class_id))
            .collect();
        for (tick, amplitude) in series.iter().enumerate() {
            cancel.interrupt(format!("stopped after {done} of {total_holds} holds"))?;
            let frame = encoder.encode_tick(
                &[*amplitude],
                &[None],
                encoder_profile,
                decoder_profile.class_count,
            )?;
            runtime.submit_sensory(frame)?;
            runtime.step(config.ticks_per_sample)?;
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
                },
            ));
        }
    }

    finish_rollout(
        run_id,
        total_holds,
        predictions,
        scored_predictions,
        scored_targets,
        metric_pack,
        events,
    )
}

fn finish_rollout<M: MetricPackPlugin>(
    run_id: &RunId,
    total_samples: u64,
    predictions: Vec<PredictionRecord>,
    scored_predictions: Vec<crate::contracts::TypedPrediction>,
    scored_targets: Vec<TypedTarget>,
    metric_pack: &M,
    events: &mut dyn RunEventSink,
) -> Result<RolloutOutcome, TrainerError> {
    let metric_result = evaluate_or_empty(metric_pack, &scored_predictions, &scored_targets)?;
    events.emit(RunEvent::new(
        run_id.clone(),
        RunEventKind::MetricUpdate {
            scope: MetricScope::Aggregate,
            metrics: metric_result.metrics.clone(),
        },
    ));
    let summary = RunSummary {
        schema_version: RUN_SUMMARY_SCHEMA_VERSION,
        run_id: run_id.clone(),
        status: RunStatus::Completed,
        total_samples,
        evaluated_samples: scored_predictions.len() as u64,
        metrics: metric_result.metrics.clone(),
        started_at: None,
        completed_at: None,
        scorecard_id: None,
        metadata: BTreeMap::new(),
    };
    Ok(RolloutOutcome {
        summary,
        predictions,
        metric_result,
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
            let class_id = scores
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).expect("ord"))
                .map(|(i, _)| i as u32)
                .unwrap_or(0);
            Ok(vec![TypedPrediction::Class { class_id, scores }])
        }
    }

    fn stream_profile(mode: StreamMode) -> EncoderBindingProfile {
        EncoderBindingProfile {
            cortical_area_id: "misc_amp".to_string(),
            channels: 1,
            scheme: EncodingScheme::Value,
            image_width: None,
            image_height: None,
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
        }
    }

    fn decoder_profile() -> DecoderBindingProfile {
        DecoderBindingProfile {
            cortical_area_id: "misc_opu".to_string(),
            class_count: 3,
            bins: 1,
            mask_width: None,
            mask_height: None,
            mask_depth: None,
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
            },
            &mut sink,
            &CancelToken::new(),
        )
        .expect("rollout");
        assert_eq!(runtime.burst_count(), 6);
        assert_eq!(runtime.motor_collects(), 0);
        assert!(runtime.submitted_rewards().is_empty());
        assert!(outcome.predictions.is_empty());
        assert_eq!(outcome.summary.evaluated_samples, 0);
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
            },
            &mut sink,
            &CancelToken::new(),
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
            },
            &mut sink,
            &CancelToken::new(),
        )
        .expect("rollout");
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
            },
            &mut sink,
            &CancelToken::new(),
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
}
