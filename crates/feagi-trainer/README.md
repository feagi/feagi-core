# feagi-trainer

Open-source (Apache-2.0) train / evaluate / benchmark **engine** for FEAGI spiking-neural
brains. It ingests datasets, runs deterministic train/test against FEAGI, and produces
verifiable benchmark records ("Scorecards").

This crate is the engine/library only. The closed-source "FEAGI Trainer" app (in
feagi-desktop) wraps this crate and owns UI, experiment infrastructure, Composer sync,
Brain Hub, scorecard publishing, and competitions. Dependency direction is strictly
one-way: closed consumes open. This crate carries **no proprietary dependencies, no UI
code, and makes no Composer/cloud calls**.

See `docs/FEAGI_TRAINER_ARCHITECTURE_AND_DESIGN.md` and `docs/FEAGI_TRAINER_ADR_SET.md`
for the full design and ADRs (ADR-001..006, ADR-012).

## Status

Initial vertical slice (IRIS tabular classification). Current scope:

- `contracts`: the public, versioned data contracts (the stable seam). v1 types:
  `DatasetManifest`, `IRSample`, `RunSpec`, `Scorecard`, `EvaluationSpec`,
  `PredictionRecord`, `RunSummary`. (`RunSpec`/`Scorecard` carry an optional
  `QuantizationFingerprint`, forward-compatible with the quantization-capable NPU direction.)
- `plugins`: the pure (non-runtime) plugin-axis interfaces — `AdapterPlugin`,
  `SamplerPlugin`, `MetricPackPlugin`.
- `adapters` / `samplers` / `metrics`: concrete implementations for the IRIS path —
  `TabularCsvAdapter`, `SequentialSampler`, `ClassificationMetricPack`.

The FEAGI binding selectors (`EncoderPlugin` / `DecoderPlugin`), `RewardPolicy`, and run
execution land next, behind a Trainer-owned runtime abstraction (remote/ZMQ path first,
embedded `feagi-npu` later once the NPU/quantization refactor stabilizes).

## Contract versioning

Contracts are versioned on two independent axes:

- `schema_version` (per contract) — the wire/format version. Bumped on breaking format
  changes; additive evolution is preferred.
- crate semver — the Rust API version.

## Layout

```text
src/
  lib.rs                  crate root
  main.rs                 thin CLI binary
  error.rs                TrainerError
  contracts/
    common.rs             shared IDs, hashes, plugin refs, taxonomies, quantization fp
    dataset_manifest.rs   DatasetManifest
    ir_sample.rs          IRSample + typed target / output-type taxonomy
    run_spec.rs           RunSpec (pinned binding, reward policy, eval protocol)
    evaluation_spec.rs    EvaluationSpec
    prediction_record.rs  PredictionRecord + TypedPrediction
    run_summary.rs        RunSummary + RunStatus
    scorecard.rs          Scorecard (status + visibility)
  plugins/                pure plugin-axis traits (adapter, sampler, metric_pack)
  adapters/tabular_csv.rs TabularCsvAdapter
  samplers/sequential.rs  SequentialSampler
  metrics/classification.rs ClassificationMetricPack
tests/
  contracts_roundtrip.rs  serde round-trip integration tests
  iris_pipeline.rs        adapter -> sampler -> metric pack integration
```
