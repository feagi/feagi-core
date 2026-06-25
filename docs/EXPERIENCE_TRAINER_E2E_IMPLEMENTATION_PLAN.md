# Experience Capture + FEAGI Trainer + Composer — Phased End-to-End Implementation Plan

Status: Proposed
Date: 2026-06-08
Owners: Neuraville Product and Architecture Working Group
Related: `docs/EXPERIENCE_CAPTURE_ARCHITECTURE_AND_DESIGN.md`, `docs/EXPERIENCE_CAPTURE_ADR_SET.md`, `docs/FEAGI_TRAINER_ARCHITECTURE_AND_DESIGN.md`, `docs/FEAGI_TRAINER_ADR_SET.md`, crate `feagi-core/crates/feagi-trainer`, app `nrs-composer`

This plan sequences the work to reach one functional end-to-end (E2E) milestone that exercises FEAGI Trainer, Experience Capture, and the minimum Composer surface together. It is grounded in the current code, not just the design docs.

---

## 1. Purpose and Scope

Deliver a thin but complete vertical slice ("walking skeleton") in which a live embodied experience is captured, packaged, offloaded to Composer, imported by FEAGI Trainer, run against a live FEAGI, scored, and the verifiable result registered back in Composer against its embodiment.

Out of scope for the skeleton: UI surfaces, catalog/search, multi-domain support, the embedded `feagi-npu` runtime, and the Nano deployment profile. These are explicitly deferred (Section 7).

---

## 2. Current Implementation State (grounded)

### 2.1 FEAGI Trainer (`feagi-core/crates/feagi-trainer`)

> **Implementation status (updated 2026-06-09):** Phase 1a, 1b, 1c, and 1e are implemented and
> tested; Phase 1d is the only remaining Phase 1 item. See the per-step markers in Section 5.

Done and tested:

- v1 public contracts: `DatasetManifest`, `IRSample`, `RunSpec`, `EvaluationSpec`, `PredictionRecord`, `RunSummary`, `Scorecard`, plus shared primitives. **(Phase 1a)** the shared primitives are now extracted into the standalone `feagi-dataset-contracts` crate (`DatasetAssetId`, `DatasetVersionId`, `ContentHash`, `Modality`, `OutputType`, `PluginRef`, `SplitId`, `ConnectomeHash`, `QuantizationFingerprint`), re-exported through `contracts/common.rs`; `feagi-trainer` depends on it. Serde round-trip covered by `tests/contracts_roundtrip.rs`.
- Pure data-pipeline plugin axes: `AdapterPlugin`, `SamplerPlugin`, `MetricPackPlugin` with concrete IRIS implementations (`TabularCsvAdapter`, `SequentialSampler`, `ClassificationMetricPack`). Composed end-to-end in `tests/iris_pipeline.rs` with synthetic predictions.
- **(Phase 1b, runtime contract)** `FeagiRuntime` trait extended with `submit_reward` and a *reserved* `submit_target_motor` teaching channel (default `Unsupported`, for the Phase 5 imitation mode); `TrainerError::Unsupported`/`Runtime` added. A deterministic in-process `StubFeagiRuntime` exercises the loop seam.
- **(Phase 1b, concrete runtime)** `RemoteFeagiRuntime` drives a live FEAGI over ZMQ via `feagi-agent` (behind the `remote-runtime` feature): registration, pre-encoded sensory + affect-channel reward publish, wall-clock `step` (Option A — the burst engine is free-running and cannot be client-stepped), and freshest-frame motor collect. Live smoke test (`tests/remote_runtime_live.rs`) self-skips without an operator-supplied endpoint.
- **(Phase 1c, run executor)** `executor::run_rollout` orchestrates encode → submit → step → collect → decode → reward → metric → `RunSummary`; `executor::assemble_scorecard` maps `RunSpec` + metrics + explicit provenance → `Scorecard`. Unit-tested plus an integration test driving the stub end-to-end (`tests/executor_rollout.rs`).
- **(Phase 1e, CLI)** `run_config::RunConfig` bundle (run spec + dataset source + binding profiles + executor cfg + scorecard provenance) with exact-id selector resolution (unknown selector = explicit error, no fallback), runtime-independent ingest/plan, and manifest-derived scorecard provenance. `feagi-trainer run --config <path> [--out <path>]` plans offline and (with `remote-runtime`) drives the live rollout and emits a `Scorecard`; endpoints are read from the environment at execution time (never persisted in provenance).

Verified to the transport boundary: the CLI connects, registers, publishes, and steps a live FEAGI. Producing a *finished* closed-loop `Scorecard` additionally needs the matching genome provisioned (below).

Still pending:

- **(Phase 1d, re-scoped)** Episodic-control metric pack + `EpisodeTrajectory` are built and tested. The closed-loop topology has been **re-scoped per ADR-014/ADR-015**: the Trainer runs as a *parallel FEAGI co-agent* (it does not drive the sim), and the embodied path is sequenced **after** the dataset path. The Topology-C engine pieces (`Environment` seam, `run_control_rollout`, env-sourced `SurvivalReward`) are **parked**. See the re-scoped Phase 1d subsection below.
- A pinned IPU/OPU genome (IRIS slice / pendulum) **provisioned into the live FEAGI** so a real motor frame is produced; without it `collect_motor` correctly times out. The MCP now exposes `load_genome_from_file(path)` to provision a genome by path (file read + upload happen server-side, keeping the genome JSON out of the model context).
- No Experience Dataset Package adapter; no `check_dataset_compatibility` (Phase 3). Experience Capture (Phase 2) and the Composer objects (Phase 4) are unstarted.

### 2.2 Experience Capture

Nothing implemented. Design and ADRs only. Requires the new open-source crate `feagi-experience-capture`.

### 2.3 Composer (`nrs-composer`, FastAPI + MongoDB)

- Has first-class `experiment`, `genome`, `embodiment`, `feagi_session`, `hub`, `marketplace`, `robot_registration` models/routers.
- `experiment` (`models/experiment.py`) is the precedent for a first-class object: prefixed id (`ex...`), `embodiment_id` foreign key, genome + session linkage.
- Embodiments: `embodiment_id` is frozen (update blacklist) but the capability spec is editable in place. Controllers are content-hash-versioned (`controller_versions`); embodiments are not.
- No dataset/experience/scorecard object yet.

---

## 3. Phase 0 — Locked Decisions

1. **Shared contracts → Option B.** Extract `feagi-dataset-contracts` (seeded from `feagi-trainer/src/contracts/common.rs`). Both `feagi-trainer` and `feagi-experience-capture` depend on it; neither pulls the other's engine. Rationale: the Nano deployment profile must stay lean, so `feagi-experience-capture` must not depend on the full Trainer crate. Cheap now (the primitives are serde-only and already isolated).
2. **Embodiment binding → coarse model key + provenance snapshot.** `embodiment_id` anchors the model; the Experience snapshots the capabilities/controller version actually used as provenance. No Composer immutability enforcement (consistent with ADR-009, generalization-first).
3. **Experience granularity → episode-level Experience + dataset bundle.** `Experience` (episode, `embodiment_id` FK) is the granular catalog record; the trainable **Dataset** (`dataset_asset_id`) bundles experiences and references their `experience_id`s. `experience_id` is distinct from `dataset_asset_id`.
4. **First slice → `mujoco_inverted_pendulum`.** Simulator-backed, ships with a genome and the `balance_homeostatic` personality. Requires a small episodic control metric pack (balance-duration / mean-reward) and a continuous motor decoder.

---

## 4. The End-to-End Milestone (M4)

> Capture a short live episode from `mujoco_inverted_pendulum` → emit a valid Experience Dataset Package → offload it to Composer → Trainer resolves the `dataset_asset_id` and imports it → runs it against a live FEAGI → produces a verifiable `Scorecard` → registers the Scorecard in Composer, tied to `embodiment_id`.

---

## 5. Phased Plan

### Phase 1 — Trainer execution path (critical path, no external dependencies)

- **1a. Extract `feagi-dataset-contracts`** and repoint `feagi-trainer` at it. Pure refactor; covered by existing `contracts_roundtrip` tests. — **DONE.**
- **1b. Concrete `FeagiRuntime`** over the remote/ZMQ `feagi-agent` path. **Design-now:** the runtime/binding contract must anticipate a future *teaching / target-motor* channel for imitation (Section 5.6), even though only the reward path is implemented here — so the loop seam is not reopened later. — **DONE** (`RemoteFeagiRuntime` + `StubFeagiRuntime`; reserved `submit_target_motor`). Best-effort wall-clock `step` only; benchmark-grade determinism awaits the embedded runtime.
- **1c. Run executor**: `RunSpec → adapter → sampler → encoder → runtime(submit/step/collect) → decoder → reward → metric pack → RunSummary → Scorecard`. — **DONE** (`executor::run_rollout` + `assemble_scorecard`).
- **1d. Episodic-control metric pack (+ continuous motor decoder)**, measuring episodic success (e.g. balance duration), not offline prediction accuracy — see Section 5.6 on embodied Scorecard semantics. — **RE-SCOPED** (ADR-014/ADR-015): the metric pack + `EpisodeTrajectory` are done; the embodied *execution* path is the **parallel co-agent** model, not the Trainer driving the sim, and is sequenced after the dataset path. See the re-scoped Phase 1d subsection.
- **1e. CLI**: wire `main.rs` to run a `RunSpec` from file and emit a `Scorecard`. — **DONE** (`run_config::RunConfig` bundle + `feagi-trainer run`).

Milestone **M1**: Trainer produces a verifiable Scorecard from an existing dataset against live FEAGI via closed-loop rollout. — **IN PROGRESS:** the execution path (1a–1c, 1e) is wired and verified to the transport boundary; reaching M1 needs 1d plus a matching genome provisioned into the live FEAGI (now provisionable via the `load_genome_from_file` MCP tool).

#### Phase 1d — Embodied execution model (re-scoped: parallel co-agent)

> Status: re-scoped 2026-06-10 by **ADR-014** (Trainer as a parallel FEAGI co-agent) and
> **ADR-015** (capture/replay boundary + embodiment-neutral contract). This supersedes the
> earlier "Topology C — Trainer drives the sim via an `Environment` seam" decision. Training
> paradigms map per scenario — see `FEAGI_TRAINER_TRAINING_PARADIGMS.md` §2 and §6.

**Execution model (decided).** On a live embodied run the Trainer is its **own independent FEAGI
agent running in parallel with the embodiment controller**, binding to *disjoint* cortical I/O:

- The **controller** (e.g. `nrs-embodiments/controllers/simulators/mujoco/`) keeps owning the
  robot's real sensory/motor and the sim physics — unchanged.
- The **Trainer agent** owns the *training-signal* I/O only: the affect/reward channel
  (Pain/Pleasure/Fear/Hope), the teaching/target-motor channel, and any goal/context input
  streams — plus readouts for scoring. The Trainer **never drives sim physics**.
- For **non-embodied datasets** there is no controller, so the Trainer is the **sole agent**
  (drives sensory + reward/pain). This is the dataset path built first (Section 5, M1).

**Reward / success ownership.** Reward is injected by the Trainer into FEAGI's **native Core
affect areas** (genome-declared task reward areas honored when present), driven by a pluggable
reward policy whose success evidence is one of: experience labels, a telemetry success predicate
(read via the neutral contract, ADR-015), or a goal-distance signal. When reward is intrinsic to
the genome (e.g. the pendulum's R-STDP `balance_homeostatic` personality), the Trainer's reward
axis is a **no-op / observe-only** and the Trainer scores rather than shapes.

**Capture / replay boundary (ADR-015).** Capture at the **cortical (FEAGI-native) boundary** is
the portable primary layer; embodiment-native traces are an opt-in `embodiment_id`-tagged
sidecar. Live "re-enactment" splits into **Mode 1 (scenario-seeding — brain attempts; supported
primary path, embodiment-agnostic)** and **Mode 2 (actuator forcing — opt-in, later, behind the
neutral contract + per-embodiment adapter)**. The Trainer never speaks an embodiment's native
command language.

**Pinned control metric (unchanged, still valid).** For control tasks the headline metric is
**mean balance/active duration** over `K` episodes, plus `success_rate`, `mean_return`, and
`duration_stddev`; `evaluation_protocol_version = ctrl-pendulum-v1`. Numeric thresholds live in
the `EvaluationSpec`/`RunConfig`, never hardcoded. The episodic-control metric pack and
`EpisodeTrajectory` implement this and survive the re-scope.

**Parked code.** The Topology-C engine pieces — `binding::environment` (`Environment` seam +
`StubEnvironment`), `run_control_rollout`, and env-sourced `SurvivalReward`, plus the
env-driving assumptions in `ContinuousMotorDecoder`/`ObservationEncoder` — are **parked** (kept
only for a possible trainer-owned, no-controller sim path). They are **not** on the live embodied
path. The episodic-control metric pack, `EpisodeTrajectory`, and Scorecard assembly remain in
active use.

**Build order (decided).** Dataset path first (sole-agent: sensory + reward/pain from labels,
credible scoring) → live embodied co-agent (observe/score + reward injection) → Experience
Capture cortical-boundary replay.

### Phase 2 — Experience Capture minimal crate (parallel with Phase 1)

- New `feagi-experience-capture`: capture-control state machine, one source connector (FEAGI bridge / MuJoCo simulator), recorder/sync, validator, packager, local store. Reuses `feagi-dataset-contracts`.
- **Design-now:** reserve the optional `language_instruction` conditioning stream in the package contract (bounded-vocabulary, faithful text + alignment, encoded at bind time — Experience Capture design Section 8.3 / ADR-010), even though the pendulum slice does not use it. Reserving it now avoids an identity-bearing schema migration when the VLA-bridge slice (Phase 5) needs it.

Milestone **M2**: capture a live episode and emit a schema-valid Experience Dataset Package locally.

### Phase 3 — Trainer ingests Experience Dataset Package

- New `ExperienceDatasetPackage` adapter in `feagi-trainer`.
- Implement `check_dataset_compatibility` (advisory/soft per ADR-009).

Milestone **M3**: captured package → Trainer run → Scorecard (Capture + Trainer integrated, local).

### Phase 4 — Minimal Composer (completes E2E)

- Composer `Experience` object (episode-level; `experience_id`, `embodiment_id` FK, capability snapshot, `content_hash`) and `Dataset` object (`dataset_asset_id`, references `experience_id`s), mirroring the `experiment` model pattern.
- Package offload/upload endpoint; dataset resolution endpoint (`dataset_asset_id` → package location); Scorecard store + read endpoint.
- Capture offloads to Composer; Trainer resolves `dataset_asset_id` from Composer; Scorecard posted back and tied to `embodiment_id`.

Milestone **M4**: the end-to-end milestone (Section 4).

### Phase 5 — VLA-bridge slice (the step from skeleton toward "FEAGI is the VLA")

The pendulum slice (M1–M4) proves the **plumbing** end-to-end but exercises none of the VLA-hard dimensions (no vision at scale, no language, no manipulation, no instructed task success). Phase 5 is the first slice that does.

- A **vision + bounded-language + manipulation** task on an arm embodiment (e.g. an instructed reach/grasp such as "pick up the red cube" on `arm_lite6` in MuJoCo).
- Exercises: a visual encoder at scale, the bounded-vocabulary `language_instruction` channel (reserved in Phase 2) end-to-end into a FEAGI language input area, a continuous high-DOF motor decoder, and **closed-loop task-success** scoring.
- Introduces the **imitation / teaching-signal** training mode against the runtime contract reserved in Phase 1b (behavior cloning from demonstrations), alongside reward.

Milestone **M5**: an instructed vision-language manipulation policy is captured, trained (imitation + reward), and scored by closed-loop task success — the first genuinely VLA-shaped slice.

### 5.6 Design-now decisions and embodied Scorecard semantics

These are decided now and reflected in Phase 1/2 scope, so the architecture is VLA-bound rather than only VLA-compatible-in-principle:

- **Embodied Scorecards mean closed-loop task success**, not offline prediction accuracy. For embodied/control datasets, a Scorecard is produced by rolling the brain out in the environment and measuring episodic success; the `EvaluationSpec` defines success and aggregation. (Offline prediction-vs-target remains valid for non-embodied datasets like IRIS.)
- **The runtime/binding contract anticipates a teaching / target-motor channel** for imitation (reserved in Phase 1b; implemented in Phase 5). Adding it later must not reopen the core loop seam.
- **The package contract reserves the `language_instruction` conditioning stream** (bounded vocabulary; reserved in Phase 2; used in Phase 5). Faithful text + alignment; bind-time encoding (ADR-010).

---

## 6. Sequencing and Parallelism

- Phase 1 is on the critical path and depends on nothing else — start immediately. 1a precedes 1b–1e.
- Phase 2 runs in parallel with Phase 1 (it does not need execution).
- Phase 3 depends on M1 + M2. Phase 4 depends on M3.
- Phase 5 (VLA-bridge) depends on M4 but is the priority follow-on; its enabling contract reservations (Phase 1b teaching channel, Phase 2 language stream) are done within the skeleton so Phase 5 adds capability without reworking contracts.

---

## 7. Deferred Beyond M4 (and M5)

- Experience Capture Nano deployment profile + store-and-forward offload (reconcile naming with existing `feagi-desktop/docs/FEAGI_NANO_IMPLEMENTATION.md`).
- Additional domains and label schemas.
- UI surfaces (Capture UI, Trainer app).
- Embedded `feagi-npu` runtime for benchmark determinism.
- Composer catalog / search / public promotion.
- **Open-vocabulary language** (MVP is bounded-vocabulary; ADR-010).
- **Cross-embodiment action retargeting** (Section 11.3 gap 5).

### 7.1 Parallel strategic track — Foundation connectome

The "FEAGI is the VLA" goal almost certainly needs a **pretrained multimodal (vision-language-action) foundation connectome** rather than growing a competent policy from scratch per task. This is the highest-leverage item for the ultimate goal and is deliberately **not** in the M1–M5 sequence; it is a parallel strategic track that should be owned and planned separately. Genome Hub already provides genome distribution, but the pretrained multimodal brain and its pretraining workflow do not yet exist.

---

## 8. Test Strategy (per phase)

- **Phase 1**: unit tests for the executor stages; an integration test driving a stub `FeagiRuntime` (deterministic) plus a live-FEAGI integration test behind a feature/marker; metric-pack correctness tests for the episodic control metric.
- **Phase 2**: connector conformance, recorder timing/quality reporting, packager golden-fixture and `content_hash` stability tests.
- **Phase 3**: adapter round-trip from a golden Experience Dataset Package; soft-compatibility tests (minute mismatch → warning, structural mismatch → block).
- **Phase 4**: Composer API tests for offload/resolve/scorecard; one E2E integration test covering M4.

Mocking is restricted to objects outside the unit under test (e.g. a stub runtime when testing the executor); the subject under test is never mocked.

---

## 9. Risks and Open Items

- **Live-FEAGI execution (1b/1c)** is the highest-risk item: ZMQ protocol/tick semantics via `feagi-agent`, and determinism of a live runtime. An embedded runtime is the long-term determinism answer but is deferred.
- **Episodic metric semantics** for the pendulum (what counts as success, aggregation window) must be pinned in the `EvaluationSpec` before M1.
- **Capability-snapshot fidelity**: ensure the Experience captures enough of the embodiment capability spec to be reproducible despite in-place embodiment edits in Composer.
- **Naming**: reconcile "Experience Capture Nano" with the pre-existing "FEAGI Nano" concept in feagi-desktop.

---

## 10. Approval Checklist

- [ ] Phase 0 decisions (Section 3) confirmed.
- [ ] First-slice `EvaluationSpec` (pendulum success/aggregation) defined before M1.
- [ ] `feagi-dataset-contracts` extraction approved (touches `feagi-trainer` contract module).
- [ ] Composer owners approve the `Experience` / `Dataset` / `Scorecard` object model and endpoints.
- [ ] Test strategy (Section 8) accepted as a per-phase gate.
- [ ] Design-now reservations accepted: teaching/target-motor channel in the runtime contract (Phase 1b) and `language_instruction` stream in the package contract (Phase 2).
- [ ] Embodied Scorecard semantics (closed-loop task success; Section 5.6) approved by Trainer owners.
- [ ] Foundation connectome (Section 7.1) assigned an owner as a parallel strategic track.

---

## 11. VLA Readiness / Gap Analysis

This section records whether the ecosystem can train a Vision-Language-Action (VLA) policy, and what is missing. It is **not** part of the E2E skeleton (Sections 4–5); it is on record so the skeleton stays scoped while the VLA path is explicit. These are deferred workstreams, currently unowned.

### 11.1 Two interpretations

- **(a) FEAGI is the VLA**: a spiking brain mapping vision + language to action, trained by experience + reward + plasticity (not backprop on a transformer). This is the native interpretation the ecosystem targets.
- **(b) FEAGI as a data factory for a conventional VLA**: FEAGI-captured trajectories train an external transformer VLA. Already supported in principle by the open Experience Dataset Package format (ADR-003); no additional work required beyond export.

The gaps below concern interpretation (a).

### 11.2 What is already covered

- Vision (V): camera/depth connectors and image/video streams.
- Action (A): command/actuator streams, episodes, `OutputType::Vector` / `Pose6Dof`, motor decoders.
- Demonstration trajectories: episode model plus teleoperation source connectors (designed).
- Embodiment grounding: `embodiment_id` plus capability snapshot; cross-embodiment provenance.
- Reward-driven (RL-style) training: the affect-channel reward axis and the `submit -> step -> collect` runtime loop.
- Reproducible scoring: Scorecards.

### 11.3 Gaps and how the plan addresses them

Status key: **contract reserved** = the seam/schema is fixed in the skeleton so no later rework; **scheduled** = implemented in a named milestone; **deferred** = acknowledged, unscheduled.

1. **Language conditioning (largest gap, both ends).** *Decision made:* bounded-vocabulary instruction channel, captured faithfully, encoded at bind time (ADR-010). *Plan:* the `language_instruction` stream is **contract-reserved** in Phase 2 and **scheduled** end-to-end in Phase 5 (a text `LanguageEncoderPlugin` into a FEAGI language input area). Open-vocabulary remains deferred.
2. **Imitation / behavior-cloning training mode (paradigm gap).** *Plan:* the runtime/binding contract **reserves** a teaching / target-motor channel in Phase 1b; the imitation mode is **scheduled** in Phase 5. Adding it does not reopen the loop seam.
3. **Closed-loop evaluation harness.** *Decision made:* embodied Scorecards mean closed-loop task success (Section 5.6). *Plan:* closed-loop rollout is **scheduled** from Phase 1 (the pendulum is already closed-loop) and extended to task-success scoring in Phase 5.
4. **Foundation connectome / pretraining.** *Plan:* **deferred** but elevated to a named parallel strategic track (Section 7.1) rather than a footnote, since it is the highest-leverage item for the ultimate goal.
5. **Cross-embodiment action retargeting.** *Plan:* **deferred** (Section 7). `embodiment_id` provenance + generalization-first exist, but no action-space normalization/retargeting layer yet.

### 11.4 Assessment

For a narrow, single-embodiment, low-data, vision-to-action policy with simple goal cues, the Capture + Trainer combo (once Phases 1–4 are built) is sufficient. Reaching a true V-L-A requires (1) language as a first-class conditioning channel end-to-end and (2) an imitation/demonstration learning mode plus closed-loop task-success eval — both now **scheduled in Phase 5** with their contracts reserved inside the skeleton (Phases 1b, 2). The foundation connectome (Section 7.1) is the remaining strategic bet. All gaps are additive; none require redesign.
