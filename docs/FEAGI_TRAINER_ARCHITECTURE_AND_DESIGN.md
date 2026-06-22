# FEAGI Trainer Architecture and Design

## 1. Purpose

This document defines the target architecture for a unified FEAGI Trainer module that allows researchers to ingest diverse machine learning datasets (images, videos, text, tabular, and multimodal), run train/test workflows against FEAGI, and produce benchmark outputs with explicit provenance.

The design prioritizes:

- Cross-modal extensibility without core rewrites.
- Deterministic input pipeline execution with explicit FEAGI-side reproducibility preconditions.
- Explicit data contracts and traceable provenance.
- Separation of concerns between ingestion, adaptation, orchestration, and evaluation.

## 2. Scope

### In scope

- Dataset ingestion and indexing.
- Dataset-to-FEAGI adaptation.
- Experiment orchestration for train/test modes.
- Evaluation and artifact generation.
- UI and API contracts for reproducible research workflows.

### Out of scope (initially)

- Automated model architecture search.
- Distributed multi-node training orchestration.
- Autonomous labeling pipelines.
- Long-term backward compatibility for legacy trainer payloads (a short migration bridge is in scope; see Section 13).

### Owned by feagi-desktop / Composer, not feagi-trainer

FEAGI Trainer is one piece that plugs into larger product surfaces. The following are explicitly **not** in feagi-trainer; the Trainer only produces the inputs they consume (e.g. `Scorecard`, `PredictionRecord`):

- Experiment infrastructure (`experiment_id`/`session_id`, lifecycle, genome auto-save/versioning).
- Brain Hub integration and genome publishing.
- Scorecard **publishing** (user-triggered, public-genome prerequisite), and Composer sync.
- Competitions, leaderboards, and scoreboards.
- Hosted dataset assets (the Trainer reserves `dataset_asset_id` and resolves locally).
- Upstream dataset **production** / live acquisition (capturing sensor/robot/biosignal streams, labeling, validation, packaging) is owned by **Experience Capture** (`feagi-experience-capture`). The Trainer is a downstream *consumer* that imports its Experience Dataset Packages through the adapter system. See `EXPERIENCE_CAPTURE_ARCHITECTURE_AND_DESIGN.md`.

See ADR-001 (responsibility boundary) and ADR-012.

This is also an **open/closed boundary**. There are two artifacts named "FEAGI Trainer": the **open-source `feagi-trainer` crate** (Apache-2.0, in feagi-core) — the data-processing/evaluation engine described by this document — and the **closed-source "FEAGI Trainer" app** (in feagi-desktop) that wraps the crate and adds UI, experiment integration, Composer sync, Brain Hub, publishing, and competitions. The dependency direction is one-way (closed consumes open); the open crate carries no proprietary dependencies. See ADR-006.

## 3. Design Principles

1. **Contract-first architecture**: all module boundaries use typed, versioned schemas.
2. **No implicit fallbacks in core flow**: invalid configuration must fail explicitly.
3. **Deterministic benchmark mode with explicit boundaries**:
   - Input-pipeline determinism: fixed seed, fixed ordering, fixed transforms.
   - FEAGI-side reproducibility is conditional on fixed genome/connectome, fixed burst/tick configuration, fixed plasticity/reward policy configuration, and deterministic runtime settings.
4. **Plugin-based extensibility (four-axis)**: the full path from raw data to FEAGI and back is extensible without core rewrites. A new dataset-and-architecture combination is supported by composing or adding plugins along four axes — **Adapter (ingest) -> Sampler -> Encoder/Decoder (FEAGI binding) -> Metric pack (evaluation)** — never by editing the orchestrator or core binding logic.
5. **Immutable run provenance**: every run records exact dataset, adapter, transforms, FEAGI config, and software versions.
6. **Separation of concerns**:
   - ingestion != mapping != orchestration != evaluation
7. **Platform agnostic**: no OS-specific assumptions in runtime behavior.

## 4. High-Level Architecture

```text
                    +---------------------------+
                    |       FEAGI Trainer UI    |
                    +-------------+-------------+
                                  |
                                  v
                    +---------------------------+
                    |  Trainer Control API      |
                    +-------------+-------------+
                                  |
                    +-------------+-------------+
                    |   Experiment Orchestrator |
                    +------+------+------+------+
                           |      |      |
                           v      v      v
                 +---------+--+ +--+-----+---------+
                 | Dataset    | | Transform Engine |
                 | Registry   | +------------------+
                 +-----+------+            |
                       |                   v
                       v         +----------------------+
              +----------------+ | FEAGI Binding Layer  |
              | Adapter Runner | +----------+-----------+
              +---+--------+---+            |
                  |        |                v
                  v        v        +--------------------+
           +------+--+ +---+------+ | FEAGI Runtime I/O  |
           | Adapters | | Samplers | +--------------------+
           +---+------+ +----------+
               |
               v
        +--------------+
        | Source Data  |
        +--------------+

                    +---------------------------+
                    | Evaluation Engine         |
                    +-------------+-------------+
                                  |
                                  v
                    +---------------------------+
                    | Run Artifact Store        |
                    +---------------------------+
```

### 4.1 Module Placement and Ownership

The Trainer is implemented as a **feagi-desktop plugin surface** with a dedicated backend service boundary:

- Frontend orchestration and researcher UX live in feagi-desktop plugin UI.
- Core Trainer control/orchestration APIs live behind a backend service boundary (the AI training service slot in desktop architecture).
- FEAGI bindings, adapter execution, and run orchestration are backend responsibilities.
- Shared UI building blocks may be reused from `feagi-react-core-pro`, but protocol/orchestration logic belongs to the Trainer service boundary.

This removes ambiguity between a pure frontend WebSocket client and a service-oriented architecture.

**Shared dataset-identity contracts.** The dataset-identity primitives currently in `feagi-trainer/src/contracts/common.rs` (`DatasetAssetId`, `DatasetVersionId`, `ContentHash`, `Modality`, `OutputType`) are also consumed by the upstream `feagi-experience-capture` crate. To avoid two divergent lineages, these primitives must be defined once and shared. Two options (see Open Decisions): (A) `feagi-experience-capture` depends on `feagi-trainer` for them, or (B) extract a small `feagi-dataset-contracts` crate that both depend on. Option B is preferred long-term so neither application crate pulls the other's engine.

## 5. Core Components

### 5.1 Dataset Registry

Maintains versioned metadata about every imported dataset:

- Dataset identity and source.
- Schema fingerprint.
- Split definitions (`train`, `val`, `test`, custom).
- Modality declarations.
- Adapter compatibility metadata.

Registry records are immutable by version. Changes create a new dataset version.

### 5.2 Adapter System

Adapters convert source formats into a canonical FEAGI Trainer Intermediate Representation (IR).

Each adapter implements:

- `discover(source_uri) -> DatasetManifestCandidate`
- `validate(manifest) -> ValidationReport`
- `map_to_ir(sample_ref) -> IRSample`
- `stream(split, sampler, transform_graph) -> Iterator[IRSample]`

Initial adapter families:

1. Image folder classification.
2. COCO-like image + annotation (for HaGRID-style workflows).
3. Tabular CSV/TSV.
4. Text corpus (classification/sequence).
5. Video + metadata annotations.
6. Experience Dataset Package (multimodal sensorimotor/signal episodes produced by Experience Capture). This adapter maps the package's stream schema, episodes, and labels into `IRSample`s; episode outcome/event labels feed a `RewardPolicy`, and embodied/control runs use the embodied/control metric pack (Section 5.8). This is additive — a new adapter in the existing plugin slot, no orchestrator or core-binding changes. See `EXPERIENCE_CAPTURE_ARCHITECTURE_AND_DESIGN.md` Section 8 for the package contract.

### 5.3 Canonical Intermediate Representation (IR)

All modalities normalize into `IRSample`:

- `sample_id: string`
- `dataset_version_id: string`
- `split: enum`
- `modality: enum(image|video|text|tabular|multimodal)`
- `payload: typed union`
- `target: TypedTarget|none` (typed by `OutputType`; see Appendix B)
- `output_type: OutputType` (the structured-output taxonomy that maps onto FEAGI's `WrappedIOType`)
- `coordinate_frame: optional` (spatial frame for dense/structured targets; see Appendix B.4)
- `timestamp: optional`
- `metadata: map<string, scalar|list>`

`target` replaces the earlier scalar-only `label`. It is a typed structure (class, bbox set, segmentation mask, keypoints, 6DOF pose, ...) selected by `output_type`. This IR decouples source data complexity from FEAGI runtime integration. See Appendix B for the structured-output taxonomy and its mapping to FEAGI's native coders.

### 5.4 Transform Engine

Runs explicit, versioned transform graphs:

- Train-only augmentations.
- Test-time deterministic preprocessing.
- Tokenization/windowing/normalization.
- Frame sampling policies.

Every transform graph is serialized into run provenance.

### 5.5 Sampler Engine

Sampling strategies are implemented as `SamplerPlugin` instances behind a common interface, so new ordering or scheduling policies can be added without modifying the orchestrator.

Each sampler implements:

- `plan(split, seed, dataset_index) -> SampleOrder`
- `next(state) -> sample_ref | end`

Built-in samplers:

- Sequential order.
- Random with seed.
- Stratified class balancing.
- Curriculum scheduling.
- Time-window replay (for sequential/video data).

Benchmark mode enforces deterministic sampling.

### 5.6 FEAGI Binding Layer

Bridges the canonical IR and the live FEAGI runtime. It contains no dataset-specific parsing logic. The binding is split into two plugin interfaces so that a new dataset architecture that requires a new encoding or motor-decoding strategy can be supported by adding plugins rather than editing core binding code.

Encoder and decoder plugins are thin **binding selectors** over FEAGI's existing coder system in `feagi-sensorimotor` (`NeuronVoxelXYZPEncoder` / `NeuronVoxelXYZPDecoder`, keyed on `WrappedIOType`). The Trainer must not build a parallel codec library; it selects, configures (via the coders' JSON properties), and records which FEAGI coder + cortical area was used. See Appendix B for the rationale and the mapping.

#### 5.6.1 Encoder plugins

`EncoderPlugin` maps an `IRSample` into a FEAGI sensory payload by selecting and configuring a FEAGI `NeuronVoxelXYZPEncoder`.

- `target_areas(binding_profile) -> [cortical_area_id]`
- `encode(ir_sample, binding_profile) -> SensoryPayload`

Responsibilities:

- Modality-to-cortical mapping resolution (IPU area targeting, voxel encoding).
- Payload encoding and tick mapping.
- Submission semantics for train vs test phase (including reward/punishment signaling where applicable).

#### 5.6.2 Decoder plugins

`DecoderPlugin` maps FEAGI motor/OPU output back into a `PredictionRecord` by selecting and configuring a FEAGI `NeuronVoxelXYZPDecoder`.

- `source_areas(binding_profile) -> [cortical_area_id]`
- `decode(motor_output, binding_profile) -> PredictionRecord`

Responsibilities:

- Class decode, action decode, or reward extraction.
- Tick synchronization with the corresponding encoded sample.
- Collection and normalization of FEAGI outputs for evaluation.

#### 5.6.3 Shared binding responsibilities

- Genome/connectome resolution for the run (the brain under test is a first-class, versioned input; see `RunSpec`).
- Tick/burst synchronization between encode submission and decode collection.
- No dataset-specific parsing logic exists in this layer.

### 5.7 Experiment Orchestrator

Creates and executes immutable `RunSpec`:

- Dataset version + split.
- Adapter version.
- Transform graph version.
- Sampler plugin + config + seed.
- Genome/connectome version (the brain under test, first-class input).
- Encoder plugin + decoder plugin versions.
- FEAGI endpoint and cortical mapping (binding) profile.
- Evaluation specification.

Manages:

- Run lifecycle (`created -> validating -> running -> completed/failed`).
- Pause/resume.
- Controlled cancellation.
- Resource usage telemetry.

### 5.8 Evaluation Engine

Pluggable metric packs:

- Classification: accuracy, precision/recall/F1, confusion matrix.
- Detection: mAP, IoU-based metrics.
- Sequence/text: exact match, macro/micro F1, BLEU/ROUGE (task-dependent).
- Time-series/tabular: MAE/MSE/R2, event-level metrics.
- Embodied/control: cumulative reward, success rate, episode length, constraint violations, recovery latency, and stability metrics.

All metrics are generated from persisted prediction records, task references, and declared evaluation protocols.

Evaluation provenance must also record:

- `evaluation_protocol_version`
- scenario/environment configuration hash (for embodied tasks)
- scorer implementation version(s)

### 5.9 Run Artifact Store

Stores:

- `RunSpec` snapshot.
- Predictions.
- Ground-truth linkage.
- Metrics and reports.
- Logs and event traces.
- Export bundles (JSON/CSV/Parquet where appropriate).

Supports experiment comparison by run ID.

## 6. API and Contract Model

### 6.1 Primary Contracts

1. `DatasetManifest`
2. `AdapterSpec`
3. `SamplerSpec`
4. `EncoderSpec`
5. `DecoderSpec`
6. `RunSpec`
7. `EvaluationSpec`
8. `PredictionRecord`
9. `RunSummary`
10. `Scorecard` (portable, verifiable score bound to a connectome/genome; see ADR-012)

All contracts must include `schema_version`.

`Scorecard` binds a `connectome_hash` (verification anchor), optional `genome_version_id`, `dataset_asset_id` + `dataset_version` + content hash, `evaluation_protocol_version`, `metric_pack` id/version, `split_id`, a backend fingerprint, the metric values, a `status` (`self_reported` | `verified`), and a `visibility` (`local` | `published`). It is a separate versioned record referencing the genome (not an in-place mutation), so a genome may carry multiple scorecards. Scorecards are generated locally automatically; **publishing is user-triggered only and requires the associated genome to be public** (see ADR-012). `DatasetManifest` reserves `dataset_asset_id` + `dataset_version`; these resolve locally today and to a hosted dataset asset in future without contract change.

### 6.2 Control API (minimum set)

- `register_dataset(source_uri, adapter_hint) -> dataset_version_id`
- `validate_dataset(dataset_version_id) -> validation_report`
- `create_run(run_spec) -> run_id`
- `start_run(run_id)`
- `pause_run(run_id)`
- `resume_run(run_id)`
- `stop_run(run_id)`
- `get_run_status(run_id) -> status`
- `get_run_metrics(run_id) -> metrics`
- `compare_runs(run_ids[]) -> comparison_report`
- `check_dataset_compatibility(dataset_manifest, stream_schema?) -> compatibility_report`

`check_dataset_compatibility` is a **pre-registration capability query**: given a (possibly external) dataset manifest and stream schema, it reports which adapter, encoder/decoder bindings, reward policies, and evaluation protocols apply, and which required fields are missing — without importing the dataset. It exposes the capability-negotiation logic already used at `validate_run` (Appendix B.5) so upstream producers such as Experience Capture can run a compatibility preflight. It is the authoritative resolver for Experience Capture's advisory `trainer_compatibility.json`. Whether this is a distinct endpoint or a generalization of `validate_dataset` is an Open Decision.

### 6.3 Transport and Runtime Boundary

- Primary control transport: backend service API (desktop invokes service commands; no direct UI-to-FEAGI protocol coupling for core runs).
- Data-plane transport to FEAGI is owned by binding plugins and FEAGI coder selection, not by UI components.
- Any direct WebSocket path used by legacy trainer modes is transitional only (see Section 13).

## 7. UI Design (Research Workflow)

### 7.1 Dataset Workflow

1. Import dataset.
2. Adapter preview and schema inspection.
3. Split configuration.
4. Label mapping verification.
5. Validation report.

### 7.2 Experiment Workflow

1. Select dataset version.
2. Select train/test split.
3. Configure transforms.
4. Configure FEAGI binding profile.
5. Configure evaluation metrics.
6. Launch run.
7. Observe live signals and final report.

### 7.3 Comparison Workflow

1. Select multiple runs.
2. Compare metrics and error slices.
3. Inspect sample-level disagreements.
4. Export benchmark summary.

### 7.4 Desktop Trainer UI (feagi-desktop plugin)

The closed **FEAGI Trainer** app (`feagi-desktop`, route `/trainer`) is a secondary Tauri window. It consumes the Control API and `RunEvent` stream only (ADR-005, ADR-011); it never opens a FEAGI socket for benchmark runs.

**Navigation model:** a **single-focus wizard** (not one scroll-everything page) with six steps:

1. **Training setup** — **`TaskTemplate`** single-select dropdown from `list_task_templates(experiment_context)` (grouped by modality family; disabled options for preview/unavailable entries with reason in detail panel); selection pre-fills one `RunConfig` draft (not parallel). Protocol template (Train → Validate → Test or subset); sampler seed and optional multi-seed repeats. CSV import is a **dataset source** on Step 2 for tabular templates. Wireframe mock: `feagi-desktop/src/plugins/trainer/taskTemplates.ts`.
2. **Dataset source** — default **Experience Catalog** tab; alternate **local package** and **import file** (pre-existing CSV) tabs. Catalog selection resolves producer-assigned `dataset_asset_id` / version / `content_hash` (ADR-012, Experience Capture ADR-006).
3. **Compatibility** — Trainer-authoritative preflight (`check_dataset_compatibility`); structural blocks vs advisory warnings (Experience Capture ADR-009 soft compatibility).
4. **Brain bindings** — encoder/decoder cortical areas, reward magnitude, ticks/sample; Advanced holds plugin ids and provenance fields.
5. **Run** — phase-aware live view driven by `RunEvent` (progress, interim/aggregate metrics, cancel).
6. **Results** — per-phase Scorecards, primary benchmark on Test phase, provenance accordion, export affordances.

**Experiment gating (browse vs run-ready):** the window **always opens**. When no active `feagi_sessions` run exists (`experimentRunSessionId` / desktop `ExperimentRunState`), the UI stays in **browse/configure** mode: setup and dataset steps remain editable; Validate, Start, and live binding probes are disabled with guidance and CTAs (Launch / Restart experiment). When the experiment stops while the Trainer is open, the UI degrades gracefully (cancel in-flight protocol, gray run actions, preserve draft config and completed scorecards). Runs and Scorecard upload require an authenticated experiment session (ADR-011/ADR-012).

**Persistent context chrome:**

- **ActiveExperimentWidget** (AppBar): embodiment thumbnail, genome title, session running indicator; popover with session/experiment ids and launch actions.
- **Precondition strip:** session, FEAGI burst, selected dataset, connectome hash status.

**Composer persistence (host policy, not crate):** a long-running training campaign on one experiment session is modeled as **`trainer_protocol`** (ordered Train/Validate/Test phases on one dataset) containing multiple **`trainer_run`** records (one crate execution per phase), each linking to a terminal **Scorecard** on `feagi_sessions.scorecards[]`. Progress snapshots for long runs are host-owned (desktop → Composer PATCH), not part of the open crate.

Wireframe implementation lives in `feagi-desktop/src/plugins/trainer/` (UI-only/mock phase until backend wiring).

### 7.5 Alignment with modern ML / MLOps workflows

The Trainer UI is **not** a gradient-descent studio (no loss curves, LR schedules, or optimizer panels — FEAGI learns via plasticity and affect channels; see `FEAGI_TRAINER_TRAINING_PARADIGMS.md`). It **is** aligned with modern **experiment tracking and benchmark** practice:

| Modern pattern | Trainer equivalent |
|---|---|
| Experiment-centric runs | Bind to live `experiment_id` + `session_id`; embodiment + genome visible in chrome |
| Dataset registry / catalog | Experience Catalog as default source; import file for legacy CSV |
| Train / val / test protocol | Explicit phase templates; Test phase = primary benchmark Scorecard |
| Preflight before execution | Compatibility step; `validate_run` / `check_dataset_compatibility` |
| Reproducibility & lineage | Scorecard pins dataset hash, protocol, connectome; session-scoped results |
| Configure → validate → run → artifacts | Six-step wizard; Run step is config-free |
| Progressive disclosure | Advanced collapsed; browse-only without live brain |

Researchers from PyTorch / Hugging Face / W&B should find the **data → protocol → run → metrics** path familiar; the **optimization** surface is intentionally different and must not be mimicked misleadingly.

### 7.6 Gaps vs best-in-class (explicit backlog)

The following gaps were identified during desktop UI design review (2026-06). They are **product/UI backlog items** for feagi-desktop + Composer unless noted as crate scope. They do **not** block the Phase 1 vertical slice (tabular IRIS + Scorecard + session bind) but affect parity with tools researchers expect (MLflow, W&B, ClearML, Roboflow, Label Studio export paths).

| Gap | Best-in-class expectation | Current design / implementation | Target phase | Owner |
|---|---|---|---|---|
| **Run comparison** | Side-by-side metrics across runs/experiments; diff configs | Results step shows one protocol only; no compare view | Phase 4 (L3) | feagi-desktop + Composer query API |
| **Live metric time series** | Charts during training (accuracy vs step), not just latest table | `RunEvent` `metric_update(partial)` rendered as table; no chart component | Phase 2 (L3) | feagi-desktop UI |
| **Experiment training history** | Dashboard of all protocols/runs on an experiment | Scorecards on `feagi_sessions`; no aggregated history UI | Phase 2 (L3) | Composer + feagi-desktop |
| **Config diff / run lineage viz** | Visual diff between RunConfigs or Scorecard provenance | Provenance accordion text-only | Phase 4 (L3) | feagi-desktop |
| **Genome/connectome snapshot UX** | Verified brain hash at run start/end; snapshot picker | `connectome_hash` placeholder; widget shows genome title only | Phase 1–2 (L4 + desktop) | feagi-core + desktop |
| **`trainer_protocol` / `trainer_run` records** | Durable multi-phase campaign with progress | Single run + scorecard attach; protocol model designed, not in Composer | Phase 2 (Composer) | nrs-composer |
| **Experience Catalog API** | Search, resolve, mount package by `dataset_asset_id` | UI designed; Composer Dataset object Phase 4 (E2E plan) | Phase 3–4 | Composer + desktop commands |
| **Plugin-contributed UI panels** | Adapter/metric panels from registry (ADR-005 §4) | Fixed wizard forms | Phase 2 (L3) | feagi-desktop + UI-contribution contract |
| **Collaboration / publish** | Share run, comment, public benchmark | Publish disabled; ADR-012 visibility lifecycle | Post-MVP | Composer |
| **Embodied / co-agent run UI** | Episode rewards, telemetry predicates, co-agent status | Paradigms 2.4 designed; not in wizard v1 | Phase 1d / 5 | feagi-desktop + crate |
| **Hyperparameter search / AutoML** | Grid search, sweeps | Out of scope (not FEAGI learning model) | N/A — non-goal | — |
| **Notebook/script-first entry** | Primary UX is code | RunConfig JSON import/export; headless CLI exists | Partial (CLI); optional export UX Phase 2 | feagi-trainer CLI + desktop |

**Highest-impact closes for “modern AI” feel without betraying FEAGI semantics:** (1) run comparison, (2) metric time-series during Run, (3) experiment training history — all listed in Phase 2–4 of Section 9.

## 8. Non-Functional Requirements

### Reliability

- Explicit failure modes with actionable validation errors.
- No silent coercion of malformed data.

### Performance

- Streaming adapters for large datasets.
- Bounded memory ingestion.
- Cached preprocessed sample chunks for repeated runs.

### Security

- File access constrained to approved paths/workspaces.
- Artifact integrity checks.
- Optional signed manifests for published benchmarks.

### Reproducibility

- Immutable run configuration.
- Full provenance capture.
- Seed + ordering persistence.
- Explicit FEAGI-side reproducibility preconditions captured in `RunSpec` and validated at run start.

## 9. Phased Delivery Plan

### Phase 1 (MVP Benchmark Foundation)

- Dataset registry.
- IR contracts.
- Adapter plugin framework.
- Adapters:
  - image folder classification
  - COCO-like image annotations
  - tabular CSV
- Deterministic run execution.
- Classification metric pack.
- Basic run comparison UI.

### Phase 2 (Multimodal Expansion)

- Text adapter.
- Video adapter with frame-window policies.
- Additional metric packs (detection, sequence).
- Better sampler policies (stratified/curriculum).
- **UI (Section 7.6):** Experience Catalog wired to Composer; live metric charts on Run step; experiment training history view; `trainer_protocol` / `trainer_run` persistence in Composer.

### Phase 3 (Research-Grade Operations)

- Advanced provenance and lineage visualization.
- Collaborative benchmark templates.
- Optional distributed run execution.
- External integrations for experiment tracking.
- **UI (Section 7.6):** plugin-contributed config/result panels (ADR-005); embodied co-agent run surfaces (ADR-014).

## 10. Risks and Mitigations

1. **Adapter sprawl**
   - Mitigation: strict adapter interface + conformance tests.
2. **Inconsistent label semantics across datasets**
   - Mitigation: explicit label-mapping step with validation gates.
3. **Non-deterministic benchmark results**
   - Mitigation: benchmark mode locks seeds/order/transform randomness and validates FEAGI-side reproducibility preconditions (genome/connectome, burst/tick config, plasticity/reward policy, runtime determinism flags).
4. **Tight coupling to FEAGI internal message format**
   - Mitigation: isolate all FEAGI protocol logic in binding layer.

## 11. Open Decisions

1. Standard artifact backend (filesystem vs object store abstraction).
2. Contract serialization format (JSON schema only vs protobuf + JSON).
3. Minimum required metric packs for first external researcher release.
4. Benchmark publishing model (local-only vs shareable signed manifests).
5. Compatibility-query surface for upstream producers: a distinct `check_dataset_compatibility` endpoint vs a generalization of `validate_dataset` to accept an unregistered manifest (Section 6.2; consumed by Experience Capture preflight).
6. Shared dataset-identity contracts placement: keep in `feagi-trainer` (Experience Capture depends on it) vs extract a `feagi-dataset-contracts` crate consumed by both (Section 4.1).
7. **`trainer_protocol` embedding:** nested under `feagi_sessions` vs top-level Composer collection for query scale (Section 7.4).
8. **Catalog default scope** on Dataset step: "My datasets" vs "Public catalog" (Section 7.4).

## 12. Immediate Next Step

Define and review the first four versioned schemas before implementation:

1. `DatasetManifest v1`
2. `IRSample v1`
3. `RunSpec v1`
4. `EvaluationSpec v1`

These schemas are the architectural backbone for all future FEAGI Trainer features.

## 13. Relationship to Existing Trainer and Migration

The existing trainer implementation under `feagi-react-core-pro/src/training/` is treated as a legacy surface.

Decision:

- **Supersede, not discard-in-place**: the new architecture replaces legacy orchestration and protocol logic.
- **Short migration bridge**: keep a time-boxed compatibility mode to preserve current workflows while new contracts are adopted.
- **No indefinite dual-stack**: legacy data/protocol compatibility does not remain open-ended.

Migration phases:

1. Introduce schema-backed `RunSpec` + dataset registry and route new runs through the new orchestrator.
2. Wrap legacy trainer actions with adapter shims where needed for temporary continuity.
3. Deprecate legacy WebSocket-only control flow after parity milestones and remove the shim path.

## Appendix A. Design Review

This appendix captures an architecture review of this document, conducted against the current FEAGI codebase (the inference engine sensory/motor I/O in `feagi-inference-engine/src/sensory_injection.rs` and `motor_extraction.rs`, the existing trainer in `feagi-react-core-pro/src/training/`, and the feagi-desktop plugin/XPC architecture in `feagi-desktop/docs/ARCHITECTURE.md`). It records the review verbatim so future contributors understand the open concerns behind the design.

### A.1 Overall Assessment

The direction is sound and the engineering discipline is strong: contract-first, no implicit fallbacks, immutable provenance, plugin-based adapters, and explicit separation of concerns. This aligns with the project's architecture rules. The majority of the design is endorsed. The reservations below concern the parts that are most FEAGI-specific, where the design is currently thinnest.

### A.2 Strengths

- **IR + adapter plugin model** (5.2/5.3) is the right abstraction and mirrors the existing plugin philosophy in feagi-desktop.
- **Determinism, provenance, and versioned schemas** match the no-fallback and reproducibility rules.
- **Phased delivery plan** is realistic, and the "define four schemas first" next step is the correct starting move.
- The design correctly avoids hardcoded endpoints. (The existing `CSVTrainer.tsx` hardcodes `127.0.0.1:9050`, which violates project rules; the new design fixes that.)

### A.3 Concerns and Disagreements

1. **The FEAGI Binding Layer (5.6) is the hardest, most novel part, yet is treated as a single box.**
   FEAGI is not a supervised gradient learner; it is a spiking, plastic network driven by a burst engine. In FEAGI terms, a "prediction" is a decoded motor/OPU fire output, a "label" is a reward/punishment or expected-motor signal, and ingestion is an encoding into IPU cortical voxels with tick synchronization (confirmed in `sensory_injection.rs` and `motor_extraction.rs`). This is the riskiest, most FEAGI-specific surface and should be the deepest section of the design, not the shallowest.

2. **The genome/connectome under test is not a first-class input.**
   A FEAGI benchmark is meaningless without specifying which brain is being tested. `RunSpec` lists a "cortical mapping profile," but the genome itself should be a versioned, provenance-captured primary artifact alongside the dataset version. Currently it is implicit.

3. **The "determinism-first benchmark" promise is overstated.**
   The design only controls the data side (seed, ordering, transforms). FEAGI-side reproducibility also depends on the genome, burst-engine configuration, plasticity/STDP, and potential GPU non-determinism. As written, the document risks promising reproducibility it cannot deliver. Either scope the claim to "deterministic input pipeline," or add explicit FEAGI-side reproducibility preconditions (fixed genome + seed + burst config + deterministic plasticity).

4. **No stated relationship to the existing trainer.**
   `feagi-react-core-pro/src/training/` already ships `Trainer`, `CSVTrainer`, `ImageTrainer`, `VideoTrainer`, `DataSourceManager`, `WebSocketManager`, and `capabilities`, with a working correct/incorrect/fitness loop. This document reads as greenfield. The out-of-scope note ("legacy trainer payload adapters") implies replacement but never states it or references the existing code. An explicit replace/wrap/supersede decision and migration note are needed.

5. **Evaluation assumes a clean classification-style ground-truth/prediction model.**
   FEAGI's heaviest use cases are embodied/control (the inverted-pendulum and MuJoCo work under `nrs-embodiments`). Continuous control and online reinforcement behavior do not fit accuracy/F1/mAP. The metric packs cover classification well, but control and sequential-reward evaluation are undefined.

6. **Placement and ownership are unspecified.**
   It is unclear whether the Trainer is a feagi-desktop plugin (Tauri/XPC, matching the "AITraining Service.xpc" slot in `ARCHITECTURE.md`), a standalone backend service, or part of feagi-react-core-pro. The Control API (6.2) looks like a backend service, whereas the current trainer is a frontend WebSocket client. This decision drives the entire API/transport design and should be settled before the schemas are written.

### A.4 Plugin-Based Extensibility Analysis

The original question is whether the architecture accommodates "a new dataset with a new architecture" via plugins. The answer is partial:

- **Pluggable today:** Adapters (5.2, per modality/format) and metric packs (5.8, explicitly "pluggable"). A new dataset *format* is cleanly handled by writing a new adapter that emits `IRSample`.
- **Not pluggable today (gap):** The **FEAGI Binding Layer (5.6)** and the **Sampler Engine (5.5)** are described as monolithic components with fixed responsibilities. A genuinely new dataset architecture frequently also requires a new **encoding strategy** (how the payload maps into cortical voxels / IPU areas) and a new **motor-decoding strategy** (how OPU fire output becomes a prediction or action). Today those would require modifying core binding logic rather than adding a plugin.

**Recommendation:** Introduce two additional plugin interfaces so the full path from raw data to FEAGI is extensible end-to-end without core rewrites:

- `EncoderPlugin`: `IRSample -> FEAGI sensory payload` (cortical-area targeting, voxel encoding, tick mapping).
- `DecoderPlugin`: `FEAGI motor/OPU output -> PredictionRecord` (class decode, action decode, reward extraction).
- Optionally promote samplers to a `SamplerPlugin` interface for the same reason.

This yields a four-axis plugin model — **Adapter (ingest) -> Sampler -> Encoder/Decoder (FEAGI binding) -> Metric pack (evaluation)** — where any new dataset-and-architecture combination is supported by composing or adding plugins, never by editing the orchestrator or core binding code.

### A.5 Recommended Actions Before Writing the Schemas

1. Add **genome version** as a first-class `RunSpec` input and provenance field.
2. Expand the **FEAGI Binding Layer** into its own detailed sub-design (encoding, motor decoding, reward signaling, tick synchronization), and split it into the `EncoderPlugin`/`DecoderPlugin` interfaces from A.4.
3. Add a **"Relationship to existing trainer"** section (replace vs. wrap, with a migration note).
4. Reframe determinism as **input-pipeline determinism** plus an explicit list of FEAGI-side reproducibility preconditions.
5. State **where the module lives** and what the Control API transport is.

## Appendix B. Future-Proofing for Structured-Output Tasks

This appendix defines how the Trainer accommodates advanced perception tasks beyond classification — **object detection, semantic segmentation, and 6DOF pose estimation** — and how it stays extensible to task types not yet anticipated.

The guiding constraint, confirmed against the codebase, is that **FEAGI already provides the structured encoding/decoding machinery**. The Trainer must reuse it, not duplicate it.

### B.1 Key Finding: FEAGI Already Has a Pluggable Coder System

The `feagi-sensorimotor` crate defines a coder abstraction keyed on a typed I/O taxonomy, and `feagi-structures` defines specialized cortical area types. The relevant existing building blocks are:

- **Coder traits** (`feagi-sensorimotor/src/neuron_voxel_coding/xyzp/coder_traits.rs`): `NeuronVoxelXYZPEncoder` and `NeuronVoxelXYZPDecoder`. Each reports its handled type via `get_encodable_data_type()` / `get_decodable_data_type()` and serializes its configuration via `JSONEncoderProperties` / `JSONDecoderProperties`.
- **Typed I/O taxonomy** (`wrapped_io_data/wrapped_io_type.rs`): `WrappedIOType` already includes `ImageFrame`, **`SegmentedImageFrame`**, **`PoseEstimationData`**, `GazeProperties`, `RawIMU`, `MiscData`, `Boolean`, and `Percentage`/`SignedPercentage` in 1D–4D variants.
- **Specialized coders already implemented**: e.g. `PoseEstimationNeuronVoxelXYZPDecoder` (spatial-cluster pose decoding), segmented image frame encoding, `cartesian_plane`, `spatial_pointer`, `positional_servo`, and `gaze_properties` decoders.
- **Specialized cortical area types** (`feagi-structures/src/genomic/cortical_area/cortical_area_type.rs`): `BrainInput` / `BrainOutput` (configured via `IOCorticalAreaConfigurationFlag`), `Custom`, `Memory`, and `Core` types — including the native affect/reward channels **Pain, Pleasure, Fear, Hope** (plus Death, Power, Fatigue).

Implication: the Trainer's `EncoderPlugin` / `DecoderPlugin` are **thin binding selectors** over these FEAGI coders. They map an IR `output_type` to a `WrappedIOType` + concrete coder + JSON properties + target cortical area, and record that selection in provenance. The Trainer does **not** implement spike-coding math.

### B.2 Output-Type Taxonomy (aligned to `WrappedIOType`)

Introduce a versioned, registry-backed `OutputType` taxonomy referenced by both `IRSample.target` and `PredictionRecord.output`. It must **map onto** FEAGI's `WrappedIOType` rather than be an independent enum:

| Trainer `OutputType` | FEAGI `WrappedIOType` | Cortical area | Status |
|---|---|---|---|
| `class` / `class_set` | `Percentage` / `Boolean` (per-class) | `BrainOutput` | exists |
| `scalar` / `vector` (regression) | `Percentage_*` / `SignedPercentage_*` | `BrainInput`/`BrainOutput` | exists |
| `segmentation_mask` | `SegmentedImageFrame` | specialized `BrainInput` | exists |
| `pose_6dof` | `PoseEstimationData` | `BrainOutput` | exists (decoder present) |
| `keypoints` | `PoseEstimationData` / `MiscData` | `BrainOutput` | partial |
| `bbox_set` (object detection) | **none yet** | `BrainOutput` (new config) | **gap — new work** |

A new task type is added by: (1) registering a new `OutputType` variant, (2) adding (or reusing) a `WrappedIOType` + coder pair in `feagi-sensorimotor`, (3) declaring the specialized cortical area config, and (4) adding a metric pack. None of these touch the orchestrator, dataset registry, or `RunSpec` schema.

### B.3 Reuse FEAGI's Native Reward Channels

Training/reward signaling must target the existing `Core` cortical areas (**Pain / Pleasure**, and optionally **Fear / Hope**) rather than inventing a Trainer-side reward mechanism. The `EncoderPlugin` for a supervised task emits the correctness/reward signal into these areas during the train phase; this is the FEAGI-native learning pathway.

### B.4 Coordinate Frame Lives in Coder Properties

Dense/spatial outputs (segmentation masks, bounding boxes, pose) require mapping FEAGI voxel activity back to input/world coordinates. This transform already lives inside the coder configuration (e.g. `SegmentedImageFrameProperties`, `PoseEstimationProperties`). The Trainer therefore:

- records the selected coder + its JSON properties in run provenance (so the spatial frame is reproducible), and
- exposes an optional `coordinate_frame` on `IRSample` / `PredictionRecord` for evaluation alignment,

rather than re-implementing coordinate transforms.

### B.5 Capability Negotiation and Conformance

As structured task types grow, validation must fail explicitly on mismatches (consistent with the no-fallback principle). At `validate_run`, the orchestrator checks a compatibility chain:

```
adapter.output_type  ==  encoder/decoder.WrappedIOType  ==  metric_pack.expects
genome BrainInput/BrainOutput areas + IOCorticalAreaConfigurationFlag  satisfy  coder requirements
coder CorticalChannelCount  ==  declared binding-profile channels
```

FEAGI already exposes the introspection needed for this (`get_encodable_data_type` / `get_decodable_data_type`, `CorticalChannelCount`, cortical area type + IO config flags). Any mismatch is a hard validation error, never a silent coercion.

### B.6 What Each Target Task Requires

- **Semantic segmentation**: reuse `SegmentedImageFrame` (encode) + the image segmentation pipeline stages; decoder reads the dense OPU grid. Metric pack: mIoU / pixel accuracy / Dice. Mostly composition of existing parts.
- **6DOF pose estimation**: reuse `PoseEstimationData` + `PoseEstimationNeuronVoxelXYZPDecoder`. Metric pack: ADD/ADD-S, rotation (geodesic) + translation error. Decoder exists; needs a pose metric pack.
- **Object detection (COCO)**: the genuine gap. Requires a new `bbox_set` `WrappedIOType` + a detection decoder (set-valued, localized) + a `BrainOutput` cortical configuration, plus an mAP/IoU metric pack. The architecture contains this work to a new coder + output-type + metric pack; it does not force core changes — but it is real research, not configuration, and should be its own milestone.

### B.7 Net Effect

The Trainer becomes a **binding-and-orchestration layer** over FEAGI's existing sensorimotor coders and cortical-area types. Future datasets and architectures are absorbed by:

1. an **Adapter** (ingest),
2. an **`OutputType`** variant mapped to a FEAGI `WrappedIOType`,
3. **Encoder/Decoder** selectors over the corresponding FEAGI coders + specialized cortical areas,
4. a **Metric pack**.

This keeps the four-axis plugin model intact while delegating the hard spike-coding and cortical-specialization concerns to FEAGI core where they already live.

## Appendix C. Points of Disagreement and Refinement

This appendix records limited disagreements/refinements relative to Appendix A/B so they are explicit and reviewable.

### C.1 OutputType mapping should be many-to-many, not interpreted as one-to-one

Appendix B's taxonomy table is directionally correct, but implementations should treat `OutputType -> WrappedIOType` as a mapping registry that can be one-to-many or many-to-one. Some tasks require composite output representations (for example, structured detections plus auxiliary attributes), and rigid one-to-one assumptions can become a scaling bottleneck.

### C.2 Reward signaling policy should be pluggable per task

Appendix B.3 correctly anchors reward signaling in FEAGI native channels (Pain/Pleasure/Fear/Hope), but the architecture should avoid prescribing one reward formulation for all tasks. Reward shaping and correctness encoding should be a configurable policy selected by binding/evaluation profiles to support supervised, weakly supervised, and reinforcement-style runs without changing core logic.

### C.3 Evaluation provenance must version protocol semantics, not only metric names

Appendix A/B emphasize metrics and coder compatibility. In addition, benchmark comparability requires versioning evaluation protocol semantics (episode definitions, aggregation windows, threshold policies, tie-break handling), because identical metric names can produce non-comparable results across protocol revisions.

## Appendix D. Response to Appendix C

This appendix responds to the disagreements/refinements raised in Appendix C. All three are **accepted** — they are refinements of Appendix A/B, not contradictions — but each requires a guardrail so it does not undermine determinism, provenance, or benchmark comparability.

### D.1 Response to C.1 (many-to-many OutputType mapping) — Accepted, with a pinning rule

Agreed. The B.2 table was illustrative, not a claim of one-to-one binding; a registry that supports one-to-many and many-to-one is correct, and composite outputs (e.g. detection = class head + box head spanning multiple cortical areas and coders) are the motivating case.

Guardrail: the mapping is many-to-many **at the registry level**, but a `RunSpec` must resolve and **pin exactly one concrete binding** (the specific set of `WrappedIOType` coders + cortical areas) per run. Otherwise provenance and determinism break, because the same `OutputType` could silently resolve to different bindings across runs. Resolution happens at `create_run`/`validate_run`; the resolved binding is frozen into the immutable `RunSpec` and recorded in provenance.

### D.2 Response to C.2 (pluggable reward policy) — Accepted, and promote it to a first-class axis

Agreed, and this is a good catch that exposes a real gap in the four-axis model. B.3 correctly anchors reward in FEAGI's native affect channels (Pain/Pleasure/Fear/Hope), but the **mapping from label/correctness/outcome to a reward signal is a policy**, and it differs across supervised, weakly-supervised, and reinforcement-style runs. It should not be hardcoded in the encoder.

Recommendation: make `RewardPolicy` a **first-class, versioned plugin/profile** (effectively a fifth axis, or an explicit sub-contract of the binding/evaluation profile), selected per run.

Guardrail: reward policy is a determinant of results, so two runs with different reward policies are **not comparable**. The reward-policy version must therefore be part of the benchmark identity / comparability key (see D.3) and captured in immutable provenance. Reward configuration must still fail explicitly (no silent default reward shaping).

### D.3 Response to C.3 (version evaluation protocol semantics) — Accepted, and enforce it in comparison

Strongly agreed; this was under-specified in A/B. Episode definitions, aggregation windows, threshold/tie-break policies, and label-matching rules must be versioned as an `evaluation_protocol_version`, not implied by metric names. This is consistent with ADR-003's implementation note.

Guardrail (extension): protocol version must be an **enforced key in run comparison**, not merely recorded. `compare_runs` must refuse — or at minimum loudly flag as non-comparable — runs that differ in `evaluation_protocol_version` (and, per D.2, in reward-policy version). Recording without enforcement still permits invalid cross-protocol benchmark claims.

### D.4 Net effect on contracts

These responses imply concrete contract changes to fold in before schema authoring:

- `RunSpec`: add a **resolved+pinned binding set**, a **`reward_policy` (versioned)** reference, and an **`evaluation_protocol_version`**.
- Provenance: record the resolved binding, reward-policy version, and evaluation-protocol version.
- `compare_runs`: treat `(evaluation_protocol_version, reward_policy_version)` as part of the comparability key and reject/flag mismatches.
- Plugin axes: recognize `RewardPolicy` as a first-class versioned plugin alongside Adapter / Sampler / Encoder+Decoder / Metric pack.

## Appendix E. Encoding-Scheme Axis and Core Coder Recommendation

This appendix records a finding about the encoder axis and a spec/recommendation to FEAGI core. It was prompted by the research requirement to let users benchmark a brain under **different neural encoding schemes** (e.g. rate, temporal, value, population). It refines ADR-002 (four-axis model) and Appendix B (coder reuse); it does not contradict them.

### E.1 Finding: encoding *scheme* is a separate dimension from data *type*, and only one scheme exists today

FEAGI's coders (`feagi-sensorimotor`) are organized by **data type** (`Percentage`, `Boolean`, `ImageFrame`, `SegmentedImageFrame`, `MiscData`, ...), keyed on `WrappedIOType`. The only neural-**encoding-scheme** controls available are:

- `NeuronDepth` — the number of bins along the z-axis of the cortical column, and
- `PercentageNeuronPositioning ∈ { Linear, Fractional }` — uniform vs exponential bin spacing.

The percentage encoder fires a voxel at the bin corresponding to the value, with the potential (P) channel hardcoded to `1.0`. In neural-coding terms this is **population / positional coding with a single spike per input dimension**, with a configurable bin count and Linear/Fractional spacing.

Mapping the commonly requested schemes against what exists (verified against `percentage_encoder.rs` and `JSONEncoderProperties`):

| Encoding scheme | Status in FEAGI today | Notes |
|---|---|---|
| Single spike, N bins (population/positional) | exists | `NeuronDepth = N`, `Linear` (uniform) or `Fractional` (exponential) bins. |
| Value / graded potential (`val`) | **gap** | Value is encoded in *position*; the P channel is hardcoded to `1.0`. A magnitude-in-potential code is not exposed. |
| Rate (value -> firing rate over ticks) | **gap** | Coders write a static voxel pattern per burst; no multi-tick rate scheduling exists. |
| Temporal / latency (value -> spike timing) | **gap** | No spike-timing/latency mechanism in the coders. |

Quantization (the `continued_npu_work` direction: `bit8/16/32/64` neuron/synapse storage) is **orthogonal** to encoding scheme — it is storage precision, not how a value becomes spikes — though bin count and value precision interact and should both be benchmarkable knobs.

### E.2 Trainer design: make encoding scheme a first-class, registered, pinned dimension

The Trainer remains a **selector** over FEAGI coders (it implements no spike-coding math, per ADR-006 / Appendix B.1). To support benchmarking across schemes:

- The `EncoderPlugin` resolves to a **pinned encoder binding** that records, as immutable provenance: the `encoding_scheme` (e.g. `population_single_spike` | `rate` | `temporal` | `value`), `bins`/`depth`, scheme parameters, the resolved concrete FEAGI coder + JSON properties + target cortical area, and the brain's `QuantizationFingerprint`. Thus `population, 8 bins, Linear, f8` is one distinct, reproducible, comparable binding.
- A **scheme registry** maps a requested `encoding_scheme` to a concrete coder + config. Today only `population_single_spike` resolves to a real coder. `rate` / `temporal` / `value` are **registered as unavailable**; `validate_run` rejects a run that selects them with an explicit error (no silent fallback to positional coding).
- The encoder scheme + version is part of the run **comparability key** (alongside `evaluation_protocol_version` and `reward_policy` version, per D.2/D.3): two runs that differ only in encoding scheme are different benchmarks and must be comparable-keyed, not silently equated.

This gives researchers the selection UX and reproducible provenance now, and absorbs new schemes without a Trainer contract change once core provides them.

### E.3 Recommendation / spec to FEAGI core (no Trainer-side spike coding)

To offer rate / temporal / value as selectable schemes, the coders must be added to `feagi-sensorimotor`. Recommendation: implement them as **quantization-aware coding schemes within the `continued_npu_work` NPU refactor**, since that branch is already reworking this exact layer. Proposed shape (additive to the existing pattern):

- Extend the coder configuration (e.g. `JSONEncoderProperties` / a parallel coding-scheme descriptor) with explicit scheme variants, each carrying its parameters and a declared quantization level:
  - `PopulationSingleSpike { bins, spacing: Linear|Fractional }` (formalizes today's behavior).
  - `Value { quantization }` — encode magnitude into the P (potential) channel rather than hardcoding `1.0`.
  - `Rate { window_ticks, max_rate, quantization }` — value -> deterministic firing count over a fixed tick window.
  - `Temporal { window_ticks, latency_map, quantization }` — value -> deterministic spike latency within a window.
- Each coder must continue to report `get_encodable_data_type()` / `get_decodable_data_type()`, serialize via JSON properties, and declare `CorticalChannelCount`, so the Trainer's capability-negotiation chain (Appendix B.5) and pinning keep working.
- **Determinism (ADR-003):** rate and temporal coders must be deterministic functions of `(value, config, tick)` and tick-synchronized with decode collection; their configuration (including quantization) must be captured in provenance. No nondeterministic spike generation in benchmark mode.

### E.4 Scope boundary

Core implements the coders; the Trainer only selects, configures (via JSON properties), pins, and records them. Until core lands rate/temporal/value, the Trainer exposes them as registered-but-unavailable and fails `validate_run` explicitly when selected.
