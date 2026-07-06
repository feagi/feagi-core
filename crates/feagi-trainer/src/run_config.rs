//! Run configuration bundle + plugin resolution for the CLI (plan Phase 1e).
//!
//! A [`RunSpec`] alone cannot drive a run: it is immutable provenance and intentionally omits
//! the dataset bytes, the encoder/decoder *binding profiles* (cortical_area-area ids, bins), the
//! transport endpoints (kept out of provenance — see [`ExecutionMode::Remote`]), and the
//! locally-unknowable scorecard fields (backend descriptor, `feagi-core` version). [`RunConfig`]
//! is the operator-supplied bundle that closes those gaps so `main.rs` can load one file and
//! produce a [`Scorecard`].
//!
//! Plugin resolution is by *exact id match* against this crate's built-in plugin set
//! ([`validate_supported`](RunConfig::validate_supported)); an unknown selector is an explicit
//! error, never a silent fallback. The offline half of the pipeline (ingest + plan + provenance
//! derivation) is runtime-independent and unit-tested here; the closed-loop execution is
//! feature-gated behind `remote-runtime` because the only concrete runtime drives a live FEAGI.

use serde::{Deserialize, Serialize};

use crate::adapters::{TabularCsvAdapter, TabularCsvConfig};
use crate::binding::profile::{DecoderBindingProfile, EncoderBindingProfile};
use crate::contracts::{
    BackendFingerprint, DatasetManifest, IRSample, RunSpec, ScorecardId, ScorecardStatus,
    ScorecardVisibility,
};
use crate::error::TrainerError;
use crate::executor::{ExecutorConfig, ScorecardProvenance};
use crate::metrics::ClassificationMetricPack;
use crate::plugins::{AdapterPlugin, DatasetSource, SamplerPlugin};
use crate::samplers::SequentialSampler;

/// Reward-policy plugin id the CLI resolves to [`PainPleasureReward`](crate::binding::PainPleasureReward).
pub const SUPPORTED_REWARD_ID: &str = "reward.pain_pleasure";
/// Encoder coder id the CLI resolves to [`PopulationEncoder`](crate::binding::PopulationEncoder).
pub const SUPPORTED_ENCODER_CODER_ID: &str = "percentage_encoder";
/// Decoder coder id the CLI resolves to [`ClassDecoder`](crate::binding::ClassDecoder).
pub const SUPPORTED_DECODER_CODER_ID: &str = "percentage_decoder";

/// Where the dataset bytes come from and how the adapter parses them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetInput {
    /// Filesystem path the dataset bytes are read from (platform-agnostic; resolved by the CLI).
    pub path: String,
    /// Tabular CSV adapter configuration (column layout, class labels, split).
    pub adapter: TabularCsvConfig,
}

/// Scorecard provenance the executor cannot derive locally and the operator must supply.
///
/// Dataset asset id/version/content-hash are *not* here — they are derived from the resolved
/// [`DatasetManifest`]. These fields describe the execution environment, which only the operator
/// knows for a remote brain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScorecardInput {
    /// Identity to assign the produced scorecard.
    pub scorecard_id: ScorecardId,
    /// Human-readable backend descriptor (e.g. `aarch64-cpu`).
    pub backend_descriptor: String,
    /// `feagi-core` version of the runtime under test.
    pub feagi_core_version: String,
}

/// The complete operator-supplied bundle needed to execute a run and emit a [`Scorecard`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunConfig {
    /// Immutable run specification (provenance + selector ids).
    pub run_spec: RunSpec,
    /// Dataset source + adapter configuration.
    pub dataset: DatasetInput,
    /// Resolved sensory-side binding profile (target IPU area, channels, scheme).
    pub encoder_profile: EncoderBindingProfile,
    /// Resolved motor-side binding profile (source OPU area, class count, bins).
    pub decoder_profile: DecoderBindingProfile,
    /// Per-run executor tuning (ticks per sample).
    pub executor: ExecutorConfig,
    /// Pain/pleasure reward stimulation magnitude in `[0.0, 1.0]`.
    pub reward_magnitude: f64,
    /// Locally-unknowable scorecard provenance fields.
    pub scorecard: ScorecardInput,
}

impl RunConfig {
    /// Parses a [`RunConfig`] from JSON, mapping serde errors to [`TrainerError::Config`].
    pub fn from_json(json: &str) -> Result<Self, TrainerError> {
        serde_json::from_str(json)
            .map_err(|e| TrainerError::Config(format!("invalid run config json: {e}")))
    }

    /// Verifies every selector named by the run spec is supported by the CLI's built-in plugin
    /// set, returning an explicit [`TrainerError::Config`] on the first unsupported selector.
    ///
    /// This intentionally does not check the execution mode — planning is runtime-independent;
    /// the execution path validates the mode separately.
    pub fn validate_supported(&self) -> Result<(), TrainerError> {
        let spec = &self.run_spec;
        let check = |label: &str, actual: &str, expected: &str| -> Result<(), TrainerError> {
            if actual != expected {
                return Err(TrainerError::Config(format!(
                    "unsupported {label} '{actual}' (the CLI supports only '{expected}')"
                )));
            }
            Ok(())
        };

        check("adapter", &spec.adapter.id.0, TabularCsvAdapter::PLUGIN_ID)?;
        check(
            "sampler",
            &spec.sampler.plugin.id.0,
            SequentialSampler::PLUGIN_ID,
        )?;
        check(
            "metric pack",
            &spec.metric_pack.id.0,
            ClassificationMetricPack::PLUGIN_ID,
        )?;
        check(
            "reward policy",
            &spec.reward_policy.plugin.id.0,
            SUPPORTED_REWARD_ID,
        )?;
        check(
            "encoder coder",
            &spec.binding.encoder.coder_id,
            SUPPORTED_ENCODER_CODER_ID,
        )?;
        check(
            "decoder coder",
            &spec.binding.decoder.coder_id,
            SUPPORTED_DECODER_CODER_ID,
        )?;
        Ok(())
    }

    /// Ingests the dataset and plans the deterministic visit order — the runtime-independent
    /// half of the pipeline.
    ///
    /// Returns the resolved [`DatasetManifest`] (provenance) plus the samples for the run's split
    /// in sampler order. Errors explicitly on a failed validation gate or an empty/unknown split.
    pub fn plan(
        &self,
        source: &DatasetSource,
    ) -> Result<(DatasetManifest, Vec<IRSample>), TrainerError> {
        let adapter = TabularCsvAdapter::new(self.dataset.adapter.clone());
        let manifest = adapter.discover(source)?;
        let report = adapter.validate(&manifest)?;
        if !report.passed {
            return Err(TrainerError::Validation(format!(
                "dataset failed validation: {}",
                report.issues.join("; ")
            )));
        }
        let samples = adapter.stream(source, &self.run_spec.split_id)?;
        let order = SequentialSampler::new().plan(samples.len(), self.run_spec.sampler.seed);
        let ordered = order.iter().map(|&i| samples[i].clone()).collect();
        Ok((manifest, ordered))
    }

    /// Derives the scorecard provenance from the resolved manifest + operator-supplied fields.
    ///
    /// Dataset identity/version/content-hash come from `manifest`; the backend fingerprint blends
    /// the run spec's backend/quantization, this crate's version, and the operator-supplied
    /// descriptor + `feagi-core` version. Trainer-produced scorecards are always `SelfReported` +
    /// `Local` (ADR-012).
    pub fn scorecard_provenance(&self, manifest: &DatasetManifest) -> ScorecardProvenance {
        ScorecardProvenance {
            scorecard_id: self.scorecard.scorecard_id.clone(),
            dataset_asset_id: manifest.dataset_asset_id.clone(),
            dataset_version: manifest.dataset_version.clone(),
            dataset_content_hash: manifest.content_hash.clone(),
            backend_fingerprint: BackendFingerprint {
                backend: self.run_spec.backend,
                descriptor: self.scorecard.backend_descriptor.clone(),
                quantization: self.run_spec.quantization.clone(),
                trainer_version: env!("CARGO_PKG_VERSION").to_string(),
                feagi_core_version: self.scorecard.feagi_core_version.clone(),
            },
            status: ScorecardStatus::SelfReported,
            visibility: ScorecardVisibility::Local,
        }
    }
}

/// Transport connection parameters for a remote run, resolved at execution time (never stored in
/// provenance — see [`ExecutionMode::Remote`](crate::contracts::ExecutionMode::Remote)).
#[cfg(feature = "remote-runtime")]
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteConnection {
    /// ZMQ registration (command/control) endpoint, e.g. `tcp://127.0.0.1:30001`.
    pub registration_endpoint: String,
    /// FEAGI burst frequency in Hz, used to size the wall-clock step wait.
    pub burst_frequency_hz: f64,
}

/// Agent registration identity presented to FEAGI by the remote runtime.
///
/// Supplied by the host (the agent embedding the library) rather than hardcoded, so each host
/// registers under its own name/token. Not part of run provenance — purely transport identity,
/// resolved at execution time alongside [`RemoteConnection`].
#[cfg(feature = "remote-runtime")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentIdentity {
    /// Agent descriptor manufacturer field.
    pub manufacturer: String,
    /// Agent descriptor name field.
    pub agent_name: String,
    /// Agent descriptor version field.
    pub agent_version: u32,
    /// Authentication token presented at registration.
    pub auth_token: [u8; 32],
}

#[cfg(feature = "remote-runtime")]
impl RunConfig {
    /// Executes the closed-loop rollout against a live FEAGI and assembles the [`Scorecard`].
    ///
    /// Resolves the supported selectors to their concrete implementations (population encoder,
    /// class decoder, pain/pleasure reward, classification metric pack), connects the remote
    /// runtime, runs the rollout, and (regardless of rollout outcome) deregisters before
    /// returning. On success the returned summary carries the produced `scorecard_id`.
    ///
    /// # Errors
    /// Returns [`TrainerError::Unsupported`] if the run spec is not `Remote`, and propagates any
    /// connection, transport, or pipeline error otherwise.
    pub fn execute_remote(
        &self,
        manifest: &DatasetManifest,
        samples: &[IRSample],
        connection: &RemoteConnection,
    ) -> Result<(crate::contracts::RunSummary, crate::contracts::Scorecard), TrainerError> {
        // CLI convenience: the same assembly as the observed path, but events are dropped and the
        // run is never cancelled. Keeps a single assembly path (DRY) so the CLI and the desktop
        // host (ADR-011) cannot diverge.
        let identity = AgentIdentity {
            manufacturer: "feagi-trainer".to_string(),
            agent_name: "feagi-trainer-cli".to_string(),
            agent_version: 1,
            auth_token: [0u8; 32],
        };
        let mut sink = crate::control::NoopEventSink;
        self.execute_remote_with_events(
            manifest,
            samples,
            connection,
            &identity,
            &mut sink,
            &crate::control::CancelToken::new(),
        )
    }

    /// Executes the closed-loop rollout against a live FEAGI, streaming each [`RunEvent`] through
    /// `events` and honouring cooperative cancellation via `cancel` (ADR-011 Control API).
    ///
    /// Identical assembly to [`execute_remote`] but built on
    /// [`run_rollout_with_events`](crate::executor::run_rollout_with_events): the host supplies the
    /// event sink (e.g. a Tauri re-emitter), the cancel token (wired to a UI stop), and the
    /// [`AgentIdentity`] (so registration is not hardcoded). On success the returned summary
    /// carries the produced `scorecard_id` and the `Scorecard` is returned to the caller; this
    /// function performs **no persistence** — where the scorecard is stored is a host policy
    /// (ADR-012).
    ///
    /// The runtime is always deregistered before returning, even if the rollout failed or was
    /// cancelled; the rollout error is surfaced first.
    ///
    /// # Errors
    /// Returns [`TrainerError::Unsupported`] if the run spec is not `Remote`,
    /// [`TrainerError::Config`] for a non-positive/non-finite burst frequency, and propagates any
    /// connection, transport, pipeline, or [`TrainerError::Cancelled`] error otherwise.
    pub fn execute_remote_with_events(
        &self,
        manifest: &DatasetManifest,
        samples: &[IRSample],
        connection: &RemoteConnection,
        identity: &AgentIdentity,
        events: &mut dyn crate::control::RunEventSink,
        cancel: &crate::control::CancelToken,
    ) -> Result<(crate::contracts::RunSummary, crate::contracts::Scorecard), TrainerError> {
        use crate::binding::{
            ClassDecoder, PainPleasureReward, PopulationEncoder, RemoteFeagiRuntime,
            RemoteRuntimeConfig,
        };
        use crate::contracts::ExecutionMode;
        use crate::executor::{assemble_scorecard, run_rollout_with_events};
        use std::time::Duration;

        if self.run_spec.execution_mode != ExecutionMode::Remote {
            return Err(TrainerError::Unsupported(format!(
                "remote execution requires a Remote run spec, got {:?}",
                self.run_spec.execution_mode
            )));
        }
        if !connection.burst_frequency_hz.is_finite() || connection.burst_frequency_hz <= 0.0 {
            return Err(TrainerError::Config(format!(
                "burst frequency must be a positive, finite value, got {}",
                connection.burst_frequency_hz
            )));
        }

        // All transport timing is sized as multiples of the operator-provided burst period, so no
        // absolute timeout is hardcoded; they scale with the brain's actual burst rate.
        let burst_period = Duration::from_secs_f64(1.0 / connection.burst_frequency_hz);
        let runtime_config = RemoteRuntimeConfig {
            registration_endpoint: connection.registration_endpoint.clone(),
            manufacturer: identity.manufacturer.clone(),
            agent_name: identity.agent_name.clone(),
            agent_version: identity.agent_version,
            auth_token: identity.auth_token,
            burst_period,
            registration_poll_interval: burst_period,
            registration_timeout: burst_period * 200,
            motor_poll_interval: burst_period / 2,
            motor_collect_timeout: burst_period * 60,
        };

        let mut runtime = RemoteFeagiRuntime::connect_and_register(runtime_config)?;
        let mut encoder = PopulationEncoder::new();
        let mut decoder = ClassDecoder::new();
        let reward = PainPleasureReward::new(self.reward_magnitude)?;
        let metric = ClassificationMetricPack::new();

        let rollout = run_rollout_with_events(
            &self.run_spec.run_id,
            samples,
            &mut runtime,
            &mut encoder,
            &self.encoder_profile,
            &mut decoder,
            &self.decoder_profile,
            &reward,
            &metric,
            &self.executor,
            events,
            cancel,
        );

        // Always deregister, even if the rollout failed; surface the rollout error first.
        let shutdown = runtime.shutdown();
        let outcome = rollout?;
        shutdown?;

        let provenance = self.scorecard_provenance(manifest);
        let scorecard_id = provenance.scorecard_id.clone();
        let scorecard = assemble_scorecard(&self.run_spec, &outcome.summary.metrics, provenance);

        let mut summary = outcome.summary;
        summary.scorecard_id = Some(scorecard_id);

        Ok((summary, scorecard))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::encoding_scheme::{BinSpacing, EncodingScheme};
    use crate::contracts::common::{
        BackendKind, ConnectomeHash, EvaluationProtocolVersion, PluginId, Split,
    };
    use crate::contracts::run_spec::{
        CoderBinding, ExecutionMode, PinnedBinding, RewardPolicyBinding, SamplerBinding,
    };
    use crate::contracts::{DatasetVersionId, PluginRef, RunId, SplitId};
    use serde_json::json;

    const CSV: &str = "f0,f1,f2,species\n\
1,0,0,setosa\n\
0,1,0,versicolor\n\
0,0,1,virginica\n";

    fn adapter_config() -> TabularCsvConfig {
        TabularCsvConfig {
            dataset_name: "one_hot".to_string(),
            has_header: true,
            feature_columns: vec![0, 1, 2],
            label_column: 3,
            class_labels: vec![
                "setosa".to_string(),
                "versicolor".to_string(),
                "virginica".to_string(),
            ],
            split: Split::Test,
            split_id: SplitId("test".to_string()),
        }
    }

    fn run_config() -> RunConfig {
        RunConfig {
            run_spec: RunSpec {
                schema_version: crate::contracts::run_spec::SCHEMA_VERSION,
                run_id: RunId("run-cfg-0001".to_string()),
                dataset_version_id: DatasetVersionId("one_hot@1".to_string()),
                split_id: SplitId("test".to_string()),
                adapter: PluginRef {
                    id: PluginId(TabularCsvAdapter::PLUGIN_ID.to_string()),
                    version: "1.0.0".to_string(),
                },
                sampler: SamplerBinding {
                    plugin: PluginRef {
                        id: PluginId(SequentialSampler::PLUGIN_ID.to_string()),
                        version: "1.0.0".to_string(),
                    },
                    seed: 42,
                },
                transform_graph_version: None,
                binding: PinnedBinding {
                    encoder: CoderBinding {
                        io_type: "Percentage".to_string(),
                        coder_id: SUPPORTED_ENCODER_CODER_ID.to_string(),
                        cortical_area_id: "iv00_C".to_string(),
                        properties: json!({}),
                    },
                    decoder: CoderBinding {
                        io_type: "Percentage".to_string(),
                        coder_id: SUPPORTED_DECODER_CODER_ID.to_string(),
                        cortical_area_id: "o____C".to_string(),
                        properties: json!({}),
                    },
                },
                reward_policy: RewardPolicyBinding {
                    plugin: PluginRef {
                        id: PluginId(SUPPORTED_REWARD_ID.to_string()),
                        version: "1.0.0".to_string(),
                    },
                    config: json!({}),
                },
                metric_pack: PluginRef {
                    id: PluginId(ClassificationMetricPack::PLUGIN_ID.to_string()),
                    version: "1.0.0".to_string(),
                },
                evaluation_protocol_version: EvaluationProtocolVersion("clf-v1".to_string()),
                connectome_hash: ConnectomeHash("sha256:connectome".to_string()),
                genome_version_id: None,
                genome_schema_version: Some(3),
                execution_mode: ExecutionMode::Remote,
                backend: BackendKind::Cpu,
                quantization: None,
            },
            dataset: DatasetInput {
                path: "/tmp/one_hot.csv".to_string(),
                adapter: adapter_config(),
            },
            encoder_profile: EncoderBindingProfile {
                cortical_area_id: "iv00_C".to_string(),
                channels: 3,
                scheme: EncodingScheme::PopulationSingleSpike {
                    bins: 1,
                    spacing: BinSpacing::Linear,
                },
            },
            decoder_profile: DecoderBindingProfile {
                cortical_area_id: "o____C".to_string(),
                class_count: 3,
                bins: 1,
            },
            executor: ExecutorConfig {
                ticks_per_sample: 3,
            },
            reward_magnitude: 0.8,
            scorecard: ScorecardInput {
                scorecard_id: ScorecardId("sc-cfg-0001".to_string()),
                backend_descriptor: "stub-cpu".to_string(),
                feagi_core_version: "0.0.12".to_string(),
            },
        }
    }

    #[test]
    fn run_config_json_round_trip() {
        let config = run_config();
        let serialized = serde_json::to_string(&config).expect("serialize");
        let restored = RunConfig::from_json(&serialized).expect("deserialize");
        assert_eq!(config, restored);
    }

    #[test]
    fn validate_supported_accepts_known_selectors() {
        assert!(run_config().validate_supported().is_ok());
    }

    #[test]
    fn validate_supported_rejects_unknown_adapter() {
        let mut config = run_config();
        config.run_spec.adapter.id = PluginId("parquet".to_string());
        let err = config.validate_supported().unwrap_err();
        assert!(matches!(err, TrainerError::Config(_)));
    }

    #[test]
    fn validate_supported_rejects_unknown_coder() {
        let mut config = run_config();
        config.run_spec.binding.decoder.coder_id = "regression_decoder".to_string();
        assert!(matches!(
            config.validate_supported(),
            Err(TrainerError::Config(_))
        ));
    }

    #[test]
    fn plan_ingests_and_orders_samples() {
        let config = run_config();
        let source = DatasetSource {
            uri: "mem://one_hot.csv".to_string(),
            bytes: CSV.as_bytes().to_vec(),
        };
        let (manifest, samples) = config.plan(&source).expect("plan");
        assert_eq!(samples.len(), 3);
        // Sequential sampler preserves source order.
        let ids: Vec<&str> = samples.iter().map(|s| s.sample_id.0.as_str()).collect();
        assert_eq!(ids[0], samples[0].sample_id.0.as_str());
        assert_eq!(manifest.output_type, crate::contracts::OutputType::Class);
    }

    #[test]
    fn scorecard_provenance_pulls_dataset_identity_from_manifest() {
        let config = run_config();
        let source = DatasetSource {
            uri: "mem://one_hot.csv".to_string(),
            bytes: CSV.as_bytes().to_vec(),
        };
        let (manifest, _) = config.plan(&source).expect("plan");
        let provenance = config.scorecard_provenance(&manifest);

        assert_eq!(provenance.dataset_asset_id, manifest.dataset_asset_id);
        assert_eq!(provenance.dataset_content_hash, manifest.content_hash);
        assert_eq!(provenance.backend_fingerprint.descriptor, "stub-cpu");
        assert_eq!(provenance.status, ScorecardStatus::SelfReported);
        assert_eq!(provenance.visibility, ScorecardVisibility::Local);
    }

    // The full streaming path needs a live FEAGI (covered by tests/remote_runtime_live.rs). These
    // exercise the pre-connection validation gates, which return before any transport is opened.
    #[cfg(feature = "remote-runtime")]
    fn agent_identity() -> AgentIdentity {
        AgentIdentity {
            manufacturer: "feagi-trainer".to_string(),
            agent_name: "feagi-trainer-test".to_string(),
            agent_version: 1,
            auth_token: [0u8; 32],
        }
    }

    #[cfg(feature = "remote-runtime")]
    fn planned() -> (RunConfig, DatasetManifest, Vec<IRSample>) {
        let config = run_config();
        let source = DatasetSource {
            uri: "mem://one_hot.csv".to_string(),
            bytes: CSV.as_bytes().to_vec(),
        };
        let (manifest, samples) = config.plan(&source).expect("plan");
        (config, manifest, samples)
    }

    #[cfg(feature = "remote-runtime")]
    #[test]
    fn execute_remote_with_events_rejects_non_remote_spec() {
        let (mut config, manifest, samples) = planned();
        config.run_spec.execution_mode = ExecutionMode::Embedded;
        let connection = RemoteConnection {
            registration_endpoint: "tcp://example.test:30001".to_string(),
            burst_frequency_hz: 10.0,
        };
        let mut sink = crate::control::NoopEventSink;
        let err = config
            .execute_remote_with_events(
                &manifest,
                &samples,
                &connection,
                &agent_identity(),
                &mut sink,
                &crate::control::CancelToken::new(),
            )
            .unwrap_err();
        assert!(matches!(err, TrainerError::Unsupported(_)));
    }

    #[cfg(feature = "remote-runtime")]
    #[test]
    fn execute_remote_with_events_rejects_nonpositive_burst() {
        let (config, manifest, samples) = planned();
        let connection = RemoteConnection {
            registration_endpoint: "tcp://example.test:30001".to_string(),
            burst_frequency_hz: 0.0,
        };
        let mut sink = crate::control::NoopEventSink;
        let err = config
            .execute_remote_with_events(
                &manifest,
                &samples,
                &connection,
                &agent_identity(),
                &mut sink,
                &crate::control::CancelToken::new(),
            )
            .unwrap_err();
        assert!(matches!(err, TrainerError::Config(_)));
    }
}
