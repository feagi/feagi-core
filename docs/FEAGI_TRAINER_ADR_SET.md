# FEAGI Trainer ADR Set

Status: Proposed  
Date: 2026-06-07  
Owners: FEAGI Trainer Architecture Working Group  
Supersedes: None  
Related: `docs/FEAGI_TRAINER_ARCHITECTURE_AND_DESIGN.md`, `docs/EXPERIENCE_CAPTURE_ARCHITECTURE_AND_DESIGN.md` (upstream dataset producer)

## ADR-001: Module Placement and Runtime Boundary

### Status
Proposed

### Context

The Trainer currently spans frontend-oriented patterns (legacy `feagi-react-core-pro` training UI/client flow) while the target architecture requires deterministic orchestration, provenance, compatibility validation, and FEAGI binding selection. These are service responsibilities, not UI responsibilities.

Without a clear boundary, protocol logic leaks into UI and creates inconsistent execution paths.

### Decision

Implement FEAGI Trainer as a `feagi-desktop` plugin surface backed by a dedicated Trainer service boundary:

- UI lives in desktop plugin windows and handles researcher interactions.
- Orchestration, validation, adapter execution, FEAGI bindings, and run artifact lifecycle live in backend service/runtime.
- UI must not own FEAGI protocol semantics for production benchmark runs.

### Responsibility boundary (Trainer vs Desktop) — clarification

FEAGI Trainer is **one piece of the puzzle**, not the home of the whole vision. The boundary is:

- **feagi-trainer (this module)** owns the **train/evaluate/benchmark engine**: adapters, samplers, encoder/decoder binding selectors, metric packs, deterministic run execution, and the **generation** of `PredictionRecord`s and local `Scorecard`s for a given dataset + connectome + protocol. It is a reusable Rust component with a Control API, usable headless.
- **feagi-desktop (and Composer)** owns the **product-level surfaces** that the Trainer plugs into: the **experiment infrastructure** (`experiment_id`/`session_id`, lifecycle, genome auto-save/versioning), **Brain Hub** integration and genome publishing, **scorecard publishing**, **competitions/leaderboards/scoreboards**, hosted **dataset assets**, Composer sync, and the cross-experiment UI.
- **Experience Capture (`feagi-experience-capture`)** owns **upstream dataset production**: live acquisition from sources/devices, labeling, validation, and packaging into Experience Dataset Packages. The Trainer is a downstream *consumer* that imports these packages via an adapter; it does not capture or label data. Producer-side decisions (capture format, label schemas, source profiles) live in `EXPERIENCE_CAPTURE_ARCHITECTURE_AND_DESIGN.md` and its own decision log, not in this ADR set.

In short: Experience Capture **produces** datasets; the Trainer **produces** verifiable results from them and plugs into experiment infra, Brain Hub, and more; the **desktop** decides how those results are published, compared, ranked, and hosted. Nothing competition/leaderboard/publishing-specific is implemented inside feagi-trainer.

This boundary is also a **licensing boundary**: the engine ships as the **open-source `feagi-trainer` crate** (Apache-2.0, in feagi-core), while the **closed-source "FEAGI Trainer" app** (in feagi-desktop) and Composer consume it one-way. See ADR-006 for the two-artifact split and the open/closed dependency invariants.

### Consequences

Positive:

- Deterministic, auditable run execution independent of UI lifecycle.
- Clear ownership and testability boundaries.
- Easier future headless/automation workflows.

Trade-offs:

- Additional service API layer to implement and maintain.
- Slightly more integration complexity versus direct UI-to-WebSocket paths.

### Alternatives Considered

1. Keep all run logic in frontend (rejected: weak reproducibility and governance).
2. Build standalone external trainer service first (rejected for now: higher integration overhead vs desktop plugin path).

### Implementation Notes

- Define `Trainer Control API` first (create/start/pause/resume/stop/status/metrics/compare).
- Route new benchmark runs exclusively through service APIs.
- Keep legacy direct-client pathways only as migration bridge (see ADR-004).

---

## ADR-002: End-to-End Plugin Model (Four-Axis Extensibility)

### Status
Proposed

### Context

Dataset diversity introduces differences at multiple layers:

- source format/parsing
- sample ordering policies
- FEAGI encoding/decoding strategy
- evaluation metrics

Adapter-only extensibility is insufficient because new task architectures often require new FEAGI binding behavior.

### Decision

Adopt a four-axis plugin model:

1. `AdapterPlugin` (ingest and IR mapping)
2. `SamplerPlugin` (ordering/scheduling)
3. `EncoderPlugin` and `DecoderPlugin` (FEAGI binding selection and runtime mapping)
4. `MetricPackPlugin` (evaluation)

The orchestrator remains stable; new dataset-and-architecture combinations are supported by plugin composition or new plugins.

### Consequences

Positive:

- Prevents orchestrator churn.
- Makes modality/task expansion predictable.
- Enables strict compatibility validation before run start.

Trade-offs:

- More plugin contracts to version and conformance-test.
- Requires plugin registry governance.

### Alternatives Considered

1. Adapter + metrics only (rejected: FEAGI binding becomes monolith).
2. Fully hardcoded FEAGI binding in orchestrator (rejected: poor scalability).

### Implementation Notes

- Encoder/Decoder plugins are binding selectors over FEAGI native coder system (`WrappedIOType`, coder traits/properties).
- Trainer does not implement parallel spike codecs.
- Add conformance tests per plugin axis and cross-axis compatibility matrix tests.

---

## ADR-003: Determinism and Reproducibility Semantics

### Status
Proposed

### Context

The phrase "deterministic benchmark mode" can be misread as end-to-end determinism. In practice, deterministic claims have two layers:

- input pipeline determinism (trainer-controlled)
- FEAGI runtime determinism (conditional on FEAGI-side constraints)

Overstating determinism creates scientific validity risk.

### Decision

Adopt explicit two-level semantics:

1. **Input-pipeline determinism (guaranteed by Trainer)**:
   - fixed dataset version
   - fixed sampler seed/order
   - fixed transform graph
2. **FEAGI-side reproducibility (validated precondition, not assumed)**:
   - fixed genome/connectome version
   - fixed burst/tick config
   - fixed plasticity/reward policy configuration
   - deterministic runtime settings and hardware constraints as declared

Runs that fail reproducibility prechecks are rejected for benchmark mode.

### Consequences

Positive:

- Correct scientific claims.
- Better comparability across runs.
- Stronger governance for published benchmarks.

Trade-offs:

- More validation complexity.
- Some environments cannot claim full benchmark-grade reproducibility.

### Alternatives Considered

1. Keep broad "deterministic benchmark" language (rejected: misleading).
2. Remove determinism goals entirely (rejected: weak benchmark credibility).

### Verified Findings (checked against feagi-core)

- **Runtime stochasticity is already deterministic by construction.** Probabilistic firing uses `excitability_random(neuron_id, burst_count)` (a PCG hash), with the *same* formula in the CPU path and the WGSL GPU shader (`burst-engine/.../neural_dynamics_fcl.wgsl`). Pattern hashing uses xxHash64 with a fixed seed 0 and explicit little-endian, order-independent input (`plasticity/src/pattern_detector.rs`). Core neuron IDs are deterministic (areas 0..=6 -> neuron IDs 0..=6). Neural dynamics is documented as "pure, deterministic, platform-agnostic" (`feagi-npu/neural/src/dynamics.rs`). There is no global runtime RNG seed to set — reproducibility is derived from `(neuron_id, burst_count)`.
- **Connectome construction is NOT reproducible.** `feagi-brain-development/src/rng.rs` exposes `get_rng()` = `rand::thread_rng()` (unseeded), used across the connectivity morphologies (projector, bitmask, patterns, etc.). Brain wiring generated from a genome therefore varies run-to-run.

### Decision refinement from findings

For benchmark mode, **pin a serialized connectome artifact** (via `feagi-serialization`) as the brain under test, rather than a genome that must be re-developed each run. This sidesteps the unseeded development RNG entirely and makes the "brain under test" bit-stable. (This refines "genome as first-class input": the pinned, provenance-captured artifact is the *post-development connectome*, optionally alongside the source genome for lineage.)

Optionally, a FEAGI-core enhancement to make `get_rng()` seedable (thread a seed from config into `StdRng`) would enable reproducible development from `genome + seed`. That is a **core change, not required for the MVP** if connectome pinning is used.

### Implementation Notes

- Benchmark mode pins a serialized connectome; record its hash in provenance.
- Add reproducibility precheck report to run artifacts (verifies pinned connectome + burst/tick config + backend).
- GPU caveat: the CPU path is the deterministic baseline; GPU floating-point reduction order may diverge. Record a backend fingerprint (CPU/GPU, driver) and require CPU (or a validated deterministic-GPU mode) for published-benchmark-grade runs.
- Capture `evaluation_protocol_version` in provenance.
- Include environment fingerprints where relevant.

---

## ADR-004: Legacy Trainer Relationship and Migration

### Status
Proposed

### Context

Existing trainer code in `feagi-react-core-pro/src/training/` provides current user workflows but does not match the target contract-first service architecture for robust benchmarking. A direct hard cut risks adoption disruption; indefinite dual-stack creates long-term architecture drift.

### Decision

Use a time-boxed supersession strategy:

- New architecture supersedes legacy orchestration/protocol paths.
- Temporary compatibility bridge is allowed to preserve active workflows.
- Legacy pathways are deprecated and removed after parity milestones.

### Consequences

Positive:

- Controlled transition with minimal user disruption.
- Prevents permanent dual architecture.
- Enables incremental rollout and validation.

Trade-offs:

- Short-term maintenance overhead for bridge components.
- Requires disciplined deprecation milestones.

### Alternatives Considered

1. Immediate hard replacement (rejected: operational risk).
2. Permanent coexistence (rejected: complexity and divergence risk).

### Implementation Notes

- Milestone M1: schema-backed `RunSpec` and dataset registry in production path.
- Milestone M2: legacy UI routes invoke new service-backed run APIs.
- Milestone M3: remove legacy direct protocol paths after feature parity and migration window.

### Naming note (product name locked: "FEAGI Trainer")

The product name is **FEAGI Trainer** (crate: `feagi-trainer`). This intentionally reuses the "trainer" name of the superseded `feagi-react-core-pro` training UI. During the migration window (M1–M3), disambiguate in docs and UI as **"FEAGI Trainer" (new, Rust, service-backed)** vs **"legacy trainer"** (`feagi-react-core-pro/src/training/`). After M3 the legacy paths are removed and the name is unambiguous.

---

## ADR-005: UI Architecture and UI-to-Service Contract

### Status
Proposed

### Context

ADR-001 pushes FEAGI protocol semantics out of the UI, but does not say how the UI is then architected or how it observes live runs. The existing trainer UI (`feagi-react-core-pro/src/training/`) is the counter-pattern: `Trainer.tsx` is a tab UI (CSV/Image/Video) that opens a **direct WebSocket to FEAGI** via `WebSocketManager`, sends capabilities, and parses raw (gzip) FEAGI frames — and it hardcodes `127.0.0.1:9050`. This couples UI to protocol, blocks headless automation, and violates the no-hardcoded-endpoint rule.

The four-axis plugin model (ADR-002) is also backend-only: there is no contract for plugins to contribute their own configuration/preview/result UI, so adding a dataset/metric today would require hand-editing core UI (the `Trainer.tsx` tabs are a fixed switch).

### Decision

Define the UI as a thin, service-driven `feagi-desktop` plugin with an explicit UI-to-service contract:

1. **Transport boundary**: the UI consumes only the Trainer Control API and a service-provided event stream. It must not open FEAGI WebSocket/ZMQ connections or parse raw FEAGI frames for benchmark runs.
2. **Live observation via normalized `RunEvent` stream**: the service owns FEAGI I/O and publishes a normalized stream (status transitions, progress, metric updates, sample-level events). The UI renders normalized events, never raw motor/sensory frames.
3. **State ownership**: the immutable `RunSpec` is service-owned source of truth. The UI holds only view/draft state. Flow is: UI draft config -> `validate_run` (service resolves + pins binding) -> immutable `RunSpec`. The UI never mutates a running `RunSpec`.
4. **Plugin-contributed UI**: each plugin axis (Adapter, Sampler, Encoder/Decoder, MetricPack, RewardPolicy) may declare optional UI surfaces (config form, preview, result view) through a declarative UI-contribution contract that mirrors the backend plugin registry. Panels read/write typed config objects validated server-side; they do not embed protocol logic.
5. **No hardcoded endpoints**: service location/config is injected (resolved via desktop config), removing the `127.0.0.1:9050` pattern.

### Consequences

Positive:

- Clean UI/service boundary; same backend serves UI and headless automation.
- Adding a dataset/metric/task contributes UI through the plugin contract, not by editing core UI.
- Reproducibility preserved because the UI cannot bypass `validate_run`/pinning.

Trade-offs:

- Requires a versioned `RunEvent` stream schema and a UI-contribution contract.
- The existing direct-WebSocket UI cannot be reused as-is; it becomes a migration source (ADR-004).

### Alternatives Considered

1. Keep direct UI-to-FEAGI WebSocket (rejected: contradicts ADR-001, blocks automation, hardcodes endpoints).
2. Fully server-rendered UI (rejected: desktop is Tauri+React; loses native plugin UX).
3. Per-dataset bespoke UI tabs as today (rejected: does not scale with the four-axis model).

### Implementation Notes

- Define `RunEvent` stream schema (status, progress, metric, sample-event) with `schema_version`; pick its transport in ADR-011 (Control API + transport).
- Define the UI-contribution contract (declarative panel descriptors + typed config payloads validated by the service).
- Migrate `Trainer.tsx` tabs to Adapter-driven panels; route run control through Control API (aligns with ADR-004 M2).
- Remove hardcoded IP/port from trainer UI as part of M2.

---

## ADR-006: Rust Implementation and feagi-core / feagi-desktop Crate Reuse

### Status
Proposed

### Context

The Trainer service (ADR-001) and its FEAGI binding (ADR-002) must run deterministically (ADR-003) and integrate with the desktop (ADR-005). FEAGI already ships the relevant capabilities as Rust crates, and `feagi-desktop/src-tauri` already depends on several of them. A Python or TypeScript backend would force reimplementation of coders, transports, and serialization, and would diverge from the Rust/RTOS migration goal.

### Decision

Implement `feagi-trainer` **fully in Rust** as a feagi-core-style crate (or small crate set), reusing existing crates rather than reimplementing. No parallel codecs, transports, or serialization.

### Two distinct artifacts and the open/closed boundary

There are **two things named "FEAGI Trainer," and they have different licenses and homes**:

1. **`feagi-trainer` (the crate)** — **open source (Apache-2.0)**, lives in the **feagi-core** workspace. The data-processing + evaluation **engine/library**: contracts, dataset registry, adapters, transforms, samplers, encoder/decoder selectors, metric packs, evaluation, scorecard **generation**, artifact format, orchestration logic, and (optionally) embedded deterministic execution.
2. **"FEAGI Trainer" (the app)** — **closed source**, an app **inside feagi-desktop**. Wraps the crate and adds the product surfaces: UI, Control API transport/service wiring, experiment infrastructure, Composer sync, Brain Hub, scorecard publishing, competitions/leaderboards.

**Dependency direction is strictly one-way:** the closed ecosystems (feagi-desktop app, Composer) depend on the open crate; the open crate **never** depends on any closed code.

Invariants for the open crate:

- Licensed **Apache-2.0**; may depend only on other feagi-core Apache-2.0 crates and public third-party crates. **No proprietary dependencies.**
- **Verified licensing:** the reuse set is all Apache-2.0 — `feagi-structures`, `feagi-sensorimotor`, `feagi-serialization`, `feagi-config`, `feagi-observability`, `feagi-agent`, `feagi-io`, and the `feagi-npu` crates (incl. `burst-engine`). So embedding the open burst engine for deterministic runs is licensing-clean.
- **`feagi-inference-engine` is proprietary and out of scope as a dependency.** The open crate may *mirror its Cargo composition pattern* (it composes the same Apache-2.0 crates) but must not link it. Any proprietary engine/online-learning features belong in the closed app, behind the crate's public API.
- The crate's **public API is the stable seam** the closed app consumes; product/integration logic stays out of the crate.
- **Contracts/schemas are open-source, public-from-day-one** in the crate (`DatasetManifest`, `IRSample`, `RunSpec`, `EvaluationSpec`, `PredictionRecord`, `RunSummary`, `Scorecard`, plugin specs, `RunEvent`). They are versioned by both `schema_version` (wire/format) and crate semver (API). Treat them as a published contract: additive evolution preferred; breaking changes bump `schema_version` and the crate major.
- **Shared dataset-identity contracts.** The dataset-identity primitives (`DatasetAssetId`, `DatasetVersionId`, `ContentHash`, `Modality`, `OutputType`, currently in `feagi-trainer/src/contracts/common.rs`) are also consumed by the upstream open crate `feagi-experience-capture`, whose Experience Dataset Package manifest is a superset of `DatasetManifest`. To avoid two divergent lineages, these primitives are defined once and shared. **Open decision (A vs B):** (A) `feagi-experience-capture` depends on `feagi-trainer` for these types, or (B) extract a small open `feagi-dataset-contracts` crate that both depend on. Option B is preferred long-term so neither application crate pulls the other's engine. Either way the open/closed and public-from-day-one invariants above apply unchanged.

Naming convention to avoid ambiguity: lowercase **`feagi-trainer`** always means the OSS crate; **"FEAGI Trainer"** (product casing) means the closed desktop app.

**No UI and no Composer/cloud I/O in the open crate.** The crate contains zero UI code and performs no calls to Composer or any cloud service. It exposes a library API and emits typed outputs (`Scorecard`, `PredictionRecord`, `RunEvent`, artifact files). The **closed app** owns all UI rendering and is the **only** component that talks to Composer (sync, publishing, experiment/session linkage, Brain Hub, leaderboards). FEAGI runtime I/O the crate does perform stays local/standard via the open `feagi-agent`/`feagi-io` transports.

Reuse map (verified to exist):

| Concern | Reused crate | Role in Trainer |
|---|---|---|
| IR data substrate | `feagi-structures` | cortical areas, XYZP neuron voxels, genomic types, errors/JSON |
| Encoder/Decoder plugins | `feagi-sensorimotor` | `NeuronVoxelXYZP{Encoder,Decoder}` + `WrappedIOType` (ADR-002 binding selectors) |
| Trainer<->FEAGI I/O | `feagi-agent` + `feagi-io` | ZMQ / WebSocket / SHM transports (ADR-011) |
| Connectome pinning + artifacts | `feagi-serialization` | serialize/load the pinned connectome (ADR-003) |
| Genome -> connectome (when building) | `feagi-evolutionary`, `feagi-brain-development` | genome parse + development |
| Embedded engine (benchmark mode) | `feagi-npu` (burst-engine/neural/runtime) | in-process, tick-locked execution |
| Config | `feagi-config` | cross-platform TOML, no hardcoded endpoints/timeouts |
| Logging | `feagi-observability` | structured tracing |
| Control API / service patterns | `feagi-api`, `feagi-services` | endpoint + service-layer patterns |
| Desktop integration | `feagi-desktop/src-tauri` | Tauri commands, `SubprocessLogRelay`, `log_manager` for any sidecar |

Two supported integration modes:

- **(a) Embedded engine** — link `feagi-npu` in-process for deterministic, tick-locked benchmark runs. Preferred for benchmark mode.
- **(b) Remote engine** — drive an existing FEAGI runtime via `feagi-agent` ZMQ. For interactive/non-benchmark use.

Reference implementation: **`feagi-inference-engine`**, which already composes `feagi-npu-burst-engine` + `feagi-io` (connectome-serialization) + `feagi-structures` + `feagi-serialization` + `feagi-observability` over ZMQ. The Trainer follows the same composition.

### Consequences

Positive:

- Aligns with the Rust/RTOS migration goal; several reused crates already expose `wasm`/`no_std` features.
- No duplicated spike codecs/transports; benchmark binding runs on tested core code.
- `feagi-desktop` already depends on `feagi-agent/io/sensorimotor/structures/serialization`, so desktop integration is incremental.

Trade-offs:

- The Trainer must track the feagi-core workspace version (currently `0.0.x`, still churning) and the desktop `[patch.crates-io]` local-path pattern.
- Embedding `feagi-npu` with the `gpu` feature pulls GPU dependencies into the Trainer build.

### Alternatives Considered

1. Python backend reusing FEAGI via FFI/REST (rejected: reimplementation, weaker determinism, off the migration path).
2. TypeScript/Node service (rejected: no access to native coders/engine; would reintroduce the protocol-in-frontend anti-pattern).

### Implementation Notes

- Start as a new crate under the feagi-core workspace (path dep), mirroring `feagi-inference-engine`'s `Cargo.toml`.
- Default to the **embedded-engine** path for benchmark runs; expose the **remote** path behind a feature/config for interactive use.
- Honor `feagi-config` for all endpoints/timeouts; no hardcoded values.
- For desktop, surface control via Tauri commands and relay any sidecar logs via `SubprocessLogRelay` (per feagi-desktop rules).

---

## ADR-011: Control API, `RunEvent` Stream, and Trainer↔Desktop Transport

### Status
Proposed

### Context

ADR-001/ADR-005 push FEAGI protocol semantics out of the UI and require the desktop to drive runs through a service boundary and observe them via a normalized event stream — but neither names the **shape of that API**, the **event schema**, or the **transport**. The legacy trainer is the counter-pattern: the React webview opens a **direct WebSocket to `ws://127.0.0.1:9050?trainer=true`** and parses raw FEAGI frames (`feagi-react-core-pro/src/training/WebSocketManager.ts`, `feagi-desktop/src/pages/Trainer.tsx`), hardcoding the endpoint and coupling UI to protocol.

Grounded findings from the current code that constrain this decision:

- **The desktop ships unsandboxed and spawns subprocesses directly; XPC was removed.** `feagi-desktop/src-tauri/entitlements.plist` says *"Direct Distribution (NOT App Store) - No Sandbox"*; `src-tauri/src/main.rs` notes *"XPC removed - using simple subprocess launching instead"*. The `AITrainingService.xpc` slot exists only in design docs (`docs/ARCHITECTURE.md`, `docs/DISTRIBUTION_STRATEGY_ASSESSMENT.md`), not in runtime code. So the App Store sandbox constraint that worried ADR-001's review is **not active today**.
- **The modern live-data pattern is already established and is the right analogue.** Rust backends own FEAGI ZMQ I/O via `feagi-agent`/`feagi-io` and emit **typed Tauri events** to React: `perception-inspector-frame`, `vision-lab-*-frame` (`feagi-desktop/src-tauri/src/plugins/{perception_inspector,vision_lab}/`, consumed via `hooks/useVisionLabFrame.ts`). This is the proven "backend owns protocol, UI renders normalized events" path.
- **Trainer scaffolding already exists**: plugin manifest `id: 'ai-training'`, route `/trainer`, `open_trainer_window` Tauri command, registry slot, and the `background_service` plugin type (`feagi-desktop/src/plugins/trainer/`, `src/plugins/types.ts`).
- **The open crate is a library + CLI**; `RunEvent` is not yet defined (`feagi-core/crates/feagi-trainer/src/contracts/mod.rs`: *"arrive with later engine/UI wiring"*).

### Decision

**1. The Control API is a Rust *library* surface in the open crate — transport-agnostic.** The crate exposes an in-process control trait/struct (create → `validate_run` → start/pause/resume/stop, plus status/metrics queries and the pre-registration `check_dataset_compatibility`) over the immutable `RunSpec`. The crate opens **no listening socket of its own**; transport is an *adapter* layered on top. This keeps the open/closed and embedded/RTOS invariants (ADR-006) intact and means the same API serves headless automation and the desktop without divergence.

**2. The normalized `RunEvent` stream is a public, versioned contract in the open crate.** A `RunEvent` (with `schema_version`) is emitted by the engine through a sink/callback the host supplies. v1 variants:
- lifecycle: `Created` / `Validating` / `Running` / `Completed` / `Failed` (mirrors `RunStatus`);
- `Progress { samples_done, samples_total, repeat_index, repeat_total }`;
- `MetricUpdate { partial | aggregate metric values }`;
- `SampleEvent { … }` (optional, sampled — never raw motor/sensory frames);
- `ScorecardReady { scorecard_id }`;
- `Error { message }`.
The UI renders these; it never sees raw FEAGI frames (ADR-005).

**3. Desktop transport = the established Tauri-events pattern (recommended).** A small **Tauri-side trainer backend** (a `background_service`-type plugin) links the crate, owns the run + FEAGI ZMQ I/O, drives the library Control API, and **re-emits each `RunEvent` as a typed Tauri event** (e.g. `trainer-run-event`) to the React window — exactly mirroring `vision-lab-*-frame`. Run control flows React → Tauri command → crate Control API. No webview-to-FEAGI socket; the legacy `9050` WebSocket path is retired (ADR-004/ADR-005).

**4. Endpoints come from config, never hardcoded.** FEAGI connection parameters resolve via `feagi-desktop/src-tauri/src/feagi_network_config.rs` / `feagi-config`; the `127.0.0.1:9050` literal and `FEAGI_TRAINER_PORT` default are removed from the trainer path.

**5. Sandbox/App-Store forward-compatibility without crate change.** Because the Control API and `RunEvent` are a library surface, a future sandboxed build hosts the *same* API inside an `AITrainingService.xpc` (or a localhost HTTP/WS micro-service) that bridges to the React window — an adapter swap, not an engine change. No work is done for this now; the decision only preserves the option.

### Consequences

Positive:

- One Control API serves desktop UI and headless/CI; reproducibility preserved because the UI cannot bypass `validate_run`/pinning.
- Reuses the proven Tauri-event live-data pattern and existing trainer scaffolding; minimal new transport surface.
- `RunEvent`/Control API stay in the open crate as versioned contracts (ADR-006), so the closed app consumes them one-way.

Trade-offs:

- Requires defining and versioning `RunEvent` + the Control API trait, and a Tauri event-bridge plugin.
- A future App Store/sandbox target still needs an XPC/localhost adapter (deferred, but the seam is reserved).

### Alternatives Considered

1. **Crate hosts its own WebSocket/HTTP server** consumed directly by the webview — rejected: reintroduces protocol-in-frontend risk, puts a network listener in the open library (against ADR-006 embedded/RTOS posture), and duplicates the existing Tauri-event mechanism.
2. **Keep the legacy direct `9050` WebSocket** — rejected: hardcoded endpoint, raw-frame parsing in UI, no headless path (contradicts ADR-001/004/005).
3. **Separate standalone trainer service process now** — rejected for the MVP: heavier ops than an in-process library + Tauri bridge, and unnecessary while the desktop is unsandboxed (revisit only for the App Store/XPC target).

### Implementation Notes

- Define `RunEvent` (+ `schema_version`) and the Control API trait in the open crate; the existing `run_rollout`/`run_repeated` executors emit events through the supplied sink.
- Add a `background_service` trainer plugin in `feagi-desktop/src-tauri` that links `feagi-trainer`, drives the Control API, relays subprocess/run logs via `SubprocessLogRelay` (per `PLUGIN_DEV_GUIDE.md`), and emits `trainer-run-event`; React subscribes with the `useVisionLabFrame`-style hook.
- Replace the legacy `Trainer.tsx`/`react-core-pro` WebSocket path during ADR-004 M2; remove the `9050` literal.
- `check_dataset_compatibility` is part of the Control API (advisory/soft per ADR-009) and serves Experience Capture preflight (design Section 6.2).
- Read-only first slice (unblocked now): a Scorecard viewer/importer needs only the existing `Scorecard` contract (incl. `metric_stats`) — no Control API — and can ship ahead of the run-control bridge.

---

## ADR-012: Genome Scorecards, Dataset Assets, and Competition Extensibility (local-first)

### Status
Proposed

### Context

The near-term goal is that a genome/connectome **published or tagged against a dataset can carry a standard, verifiable score**, so researchers can **choose** between genomes and **validate** a claimed score. Two related capabilities are explicitly **future** but must not be designed out:

- **Hosted dataset assets** with unique, versioned IDs.
- **Competitions / leaderboards**, which will be closed-source or otherwise controlled.

All datasets in scope are **public**, so hidden-label scoring provides no protection; integrity must come from **reproducibility verification** (re-running the pinned connectome). The work must be **local-first** but use IDs/schemas that later lift into Composer without rework.

### Decision

Introduce a first-class **`Scorecard`** record and reserve the dataset-asset and competition identifiers now.

**Scorecard** — a portable, verifiable benchmark result bound to a genome/connectome. It is a **separate, versioned record that references** the genome (never an in-place mutation), so one genome may carry multiple scorecards (one per dataset/protocol) and a history. A scorecard pins:

- `connectome_hash` (the pinned, re-runnable artifact — the verification anchor, per ADR-003)
- `genome_version_id` / lineage (optional, for provenance)
- `dataset_asset_id` + `dataset_version` + dataset content hash
- `evaluation_protocol_version` (ADR-010 semantics)
- `metric_pack` id + version, `split_id`
- backend fingerprint (CPU/GPU) + Trainer/feagi-core workspace version
- the metric values
- *(optional, additive)* per-metric **N-seed distribution** `metric_stats` — for runs repeated over N derived seeds, each metric carries `{n, mean, stddev, ci_low, ci_high, confidence_level}` (Student's-t interval); `metrics` then holds the per-metric means. Omitted for single runs (backward-compatible). The repeat orchestration (`stats::run_repeated` + `aggregate_metric_stats`) re-plans the sampler order per seed against the same pinned connectome — order-dependent plasticity is the genuine variance source (ADR-003).
- `status`: `self_reported` | `verified`

**Validation = re-run.** Given a scorecard's pinned connectome + dataset version + protocol, any party can re-execute and confirm the metrics within tolerance. `status` becomes `verified` when an independent re-run matches. This is the same mechanism a future competition uses for integrity.

**Publication gating.** Scorecards are **generated and stored locally automatically** (private, for the user's own comparison). **Publishing** a scorecard is a distinct, gated action:

- It happens **only when the user explicitly triggers it** — never automatically.
- It has a **hard prerequisite that the associated genome is public**. Publishing is rejected if the genome is private/unpublished (the score has no value to others if they cannot obtain or re-run the genome).

A scorecard therefore carries a `visibility` state (`local` -> `published`) separate from its `status` (`self_reported` | `verified`). Publication binds the scorecard to the public genome's identity so consumers can fetch and re-validate it.

**Dataset assets (future, IDs reserved now).** `DatasetManifest` carries a `dataset_asset_id` + `dataset_version` + content hash. For the MVP these resolve to a **local** manifest/content hash; the *same* IDs later resolve to a hosted asset with no contract change.

**Producer of dataset identity.** When a dataset originates from Experience Capture, the **producer** (Experience Capture) assigns `dataset_asset_id` / `dataset_version` and computes the content hash over the package's identity-bearing contents; the Trainer **consumes and resolves** that identity rather than minting a new one at import. A label correction in Experience Capture advances `dataset_version` + content hash, which keeps a `Scorecard`'s pinned `(dataset_asset_id, dataset_version, content_hash)` bound to fixed bytes and labels (verification-by-re-run holds). Datasets imported directly by the Trainer (not via Experience Capture) continue to resolve identity locally as before.

**Competition extensibility (future, controlled).** A competition is a controlled set of scorecards under an organizer-fixed `(dataset_version, evaluation_protocol_version, division rules)`, ranked, with reproducibility-verification as the integrity model. Nothing competition-specific is built now; the requirement is only that the `Scorecard` + comparability key (ADR-010) + `connectome_hash` carry everything a future leaderboard needs.

**Ownership (per ADR-001 boundary).** feagi-trainer only **generates** scorecards (compute the score, pin the connectome, record provenance, support local self-verify). All higher-level handling — **publishing** (user trigger + public-genome prerequisite), Brain Hub binding, **competitions/leaderboards/scoreboards**, and hosted **dataset assets** — is owned by **feagi-desktop + Composer**, which consume the Trainer's scorecards. The `visibility`/`status` fields and reserved `dataset_asset_id` exist so desktop/Composer can drive those flows without changing the Trainer.

### Consequences

Positive:

- Published/Brain-Hub genomes can advertise a standard, re-runnable score; users compare and validate without trusting the publisher.
- Local-first delivery; cloud hosting, dataset registry, and competitions are additive on the same IDs.
- Reuses the ADR-003 connectome-pinning artifact as the verification anchor.

Trade-offs:

- Requires the run-scoped genome/connectome versioning prerequisite (gap analysis Gap 2) to attach scorecards to specific snapshots rather than an overwritten genome doc.
- "Verified" status requires a re-run path (local self-verify now; trusted/cloud verification later).

### Alternatives Considered

1. Embed the score directly in the genome document (rejected: in-place overwrite loses history and per-dataset multiplicity; collides with Gap 2).
2. Hidden-label scoring for integrity (rejected: datasets are public).
3. Defer all scorecard structure until competitions exist (rejected: would force a later rework of published-genome metadata).

### Implementation Notes

- Add `Scorecard` to the primary contracts; reference it from the genome/Brain-Hub publish metadata.
- MVP: generate and store scorecards in the local artifact store; resolve `dataset_asset_id` locally; support local self-verification (re-run matches within tolerance).
- Publishing is user-triggered only; enforce the public-genome prerequisite at publish time and reject otherwise. Scorecards default to `visibility: local`.
- Keep CPU as the verification baseline (GPU fingerprint recorded; not verification-grade) per ADR-003.
- Do not build leaderboard/competition logic now; only ensure schema completeness for it.

---

## ADR-014: Trainer as a Parallel FEAGI Co-Agent (embodied training topology)

### Status
Proposed

### Context

Embodied tasks already ship a **controller that is itself a FEAGI agent** owning the robot's real sensory/motor streams and the simulator physics (e.g. the `nrs-embodiments` MuJoCo controller, which talks to FEAGI directly over ZMQ and handles episode resets via `MiscResetCommandTap`). FEAGI also runs learning *inside* the engine (plasticity / R-STDP) driven by neural co-activation and its native affect channels; the Trainer never runs a learning rule (see `FEAGI_TRAINER_TRAINING_PARADIGMS.md` §1).

An earlier Phase 1d framing (Topology C) assumed the Trainer would **drive the environment** through an additive `Environment` seam — `env.reset → submit sensory → step FEAGI → collect motor → env.step(action)`. Reviewing the existing integration showed this contends with the controller (two agents fighting over the robot's sensory/motor) and reimplements physics/mapping ownership the controller already holds. The Trainer's role on a live embodied run had to be settled.

### Decision

On a live embodied run, the Trainer participates as its **own independent FEAGI agent, running in parallel with the embodiment controller, binding to disjoint cortical I/O**:

- **The controller owns** the robot's real sensory/motor streams and the simulator physics (unchanged).
- **The Trainer owns the training-signal I/O**: the affect/reward channel (Pain/Pleasure/Fear/Hope), the teaching/target-motor channel, and any goal/context input streams (e.g. object coordinates, ideal-IMU goal), plus readouts for scoring. **The Trainer never drives sim physics.**
- For **non-embodied datasets** (e.g. cancer-cell anomaly detection, IRIS) there is no controller, so the Trainer is the **sole agent** — it drives the sensory input *and* the reward/pain.

Supporting choices ratified here:

- **Reward target.** The Trainer injects into FEAGI's **native Core affect areas (Pain/Pleasure/Fear/Hope)** as the general mechanism; genome-declared task reward areas (e.g. a `balance_reward` IPU) are honored when the genome exposes them. The Trainer never invents a side-channel reward.
- **Success evidence (pluggable reward policy).** The reward policy derives the affect signal from one of: (i) **experience labels** (per-sample/per-episode correctness — datasets, coordinate tasks), (ii) a **telemetry success predicate** over embodiment state read via the neutral contract (ADR-015) (e.g. "object grasped"), or (iii) a **goal-distance** signal (deviation from a target, e.g. ideal IMU). Reward injection is therefore a per-task policy that can also be a no-op when reward is intrinsic to the genome.
- **Episode boundaries.** The Trainer owns them — it commands `reset(scenario, seed)` and consumes `episode_started`/`episode_ended` telemetry (ADR-015), tick-clock aligned, so reward lands on the correct behavior and scoring is segmented.

### Consequences

Positive:

- Resolves the sensory/motor contention: controller and Trainer target disjoint cortical areas on the same genome, which FEAGI's multi-agent design already supports.
- Keeps the open-source crate **embodiment-agnostic** — the Trainer speaks FEAGI cortical I/O, not robot-native commands.
- One model covers every scenario in `FEAGI_TRAINER_TRAINING_PARADIGMS.md` §6 (arm pickup, coordinate→behavior, anomaly detection, quadruped) by varying only *which signals the Trainer agent injects* and *the data source*.

Trade-offs / supersession:

- **Supersedes Phase 1d Topology C.** The `Environment`-as-sim-driver code (`binding::environment` seam, `run_control_rollout`, env-sourced `SurvivalReward`, the env-driving assumptions in `ContinuousMotorDecoder`/`ObservationEncoder`) is **parked** (kept only for a possible trainer-owned, no-controller sim path; not on the live embodied path). The **episodic-control metric pack, `EpisodeTrajectory`, and Scorecard assembly remain valid** under this model.
- Requires a thin, versioned control/telemetry contract between the two agents (ADR-015).

### Alternatives Considered

1. **Observer-only** (Trainer reads streams and scores, injects nothing) — rejected: cannot deliver the reward/teaching/goal signals that *training* requires; only supports passive scoring.
2. **Trainer replaces the controller / drives the sim** (Topology C) — rejected: contends with the controller and reimplements embodiment physics/mappings the controller already owns. Retained only as a parked, no-controller sim path.

### Implementation Notes

- Near-term build order (decided): the **dataset path first** (sole-agent: drive sensory + reward/pain from labels, score credibly), then the live embodied co-agent path, then Experience Capture replay.
- The co-agent path depends on the neutral control/telemetry contract (ADR-015) and on the controller exposing episode lifecycle + minimal outcome telemetry.
- Park, do not delete, the Topology-C code; gate it behind the trainer-owned-sim use case if it is ever needed.

---

## ADR-015: Capture/Replay Boundary and Embodiment-Neutral Training Contract

### Status
Proposed

### Context

To expose a brain to captured experience on a **live** run, "re-enact the episode" hides two very different modes:

- **Mode 1 — scenario seeding.** The capture defines the *task setup* (object pose, home pose, episode seed) and the success criterion; the controller resets the sim to that setup and the **brain's own motor output** attempts the task. The capture does not move the actuators.
- **Mode 2 — demonstration forcing.** The captured **actuator trajectory is replayed** through the sim, physically moving the robot through the demonstrated motion while the brain observes (imitation/teaching).

Mode 2 with true actuator forcing requires speaking the embodiment's **native command language** (joint names, ranges, control modes). If the Trainer or Experience Capture had to speak each controller's native language, coupling would explode as `N embodiments × Trainer`.

### Decision

- **Capture at the cortical (FEAGI-native) boundary as the portable, primary layer**: per-tick cortical I/O, episode metadata (boundaries, seed, success label, reward/pain events, tick clock), and task/goal context in a **neutral schema**. Replaying neural activation needs zero knowledge of the embodiment.
- **Embodiment-native data (actuator trace, sim state, native initial conditions) is captured only as an optional, namespaced sidecar** tagged with `embodiment_id`. It is opaque to the Trainer/Experience Capture.
- **The Trainer speaks exactly two languages:** FEAGI cortical I/O (it is an agent) and a **narrow, neutral control/telemetry contract** with the controller:
  - Trainer → controller: `reset(scenario, seed)`, `start_episode`, `end_episode` (neutral scenario params).
  - controller → Trainer: `episode_started`/`episode_ended` + the minimal outcome telemetry the reward policy needs, on the tick clock.
- **All embodiment-specific translation lives in the controller's adapter**, not in the Trainer or Experience Capture. The controller already owns native sim ops (it translates a neutral `reset(scenario)` into, e.g., MuJoCo `qpos`).
- **Mode 1 (scenario-seeding, cortical-boundary) is the supported primary path** and is fully embodiment-agnostic. **Mode 2 (actuator forcing) is opt-in and later**, gated behind the neutral contract + a per-embodiment adapter that knows how to replay the native sidecar; the Trainer still only says "replay demo N".

### Consequences

Positive:

- The open-source crate and Experience Capture stay **embodiment-agnostic**; no `N×Trainer` coupling.
- Cortical-boundary replay is **deterministic and reproducible** (good for publication-credible Scorecards).
- High-fidelity native re-enactment remains available without leaking embodiment knowledge into the Trainer.

Trade-offs:

- Introduces a new **versioned neutral control/telemetry contract** to maintain (and a per-embodiment adapter for Mode 2 when it lands).
- Mode 2 fidelity depends on each controller implementing its side of the contract.

### Alternatives Considered

1. **Trainer/Experience Capture speak each controller's native language** — rejected: coupling explosion; breaks the embodiment-agnostic open-crate invariant (ADR-006).
2. **Capture only embodiment-native traces** — rejected: not portable and not replayable without the matching embodiment; defeats cross-embodiment provenance and reproducible scoring.

### Implementation Notes

- Define the neutral control/telemetry contract with a `schema_version`; place it in shared open contracts (consistent with ADR-006 / the `feagi-dataset-contracts` direction).
- Experience Capture stores the cortical-boundary streams as the trainable content and the native trace as an `embodiment_id`-tagged sidecar (reconcile with the Experience Capture package contract).
- Do not build Mode 2 actuator forcing until cortical-level teaching proves insufficient.

---

## ADR Approval Checklist

Before implementation begins, confirm:

- [ ] Architecture leads approve ADR-001..006, ADR-011, ADR-012, ADR-014, and ADR-015.
- [ ] Contract schemas (`DatasetManifest`, `IRSample`, `RunSpec`, `EvaluationSpec`) explicitly reference these ADR decisions.
- [ ] Conformance test strategy exists for all plugin axes.
- [ ] Migration milestones have target release windows and owner assignments.
- [ ] Shared dataset-identity contracts placement (Option A vs B, ADR-006) is decided jointly with Experience Capture owners.
- [ ] Experience Capture is consumed only via an Experience Dataset Package adapter (consumer side); producer-side decisions are deferred to the Experience Capture decision log (ADR-001 boundary).

---

## Appendix A. Architecture Review Feedback

This appendix records review feedback on ADR-001..004, in the same spirit as the appendices in `FEAGI_TRAINER_ARCHITECTURE_AND_DESIGN.md`. Overall: the four ADRs are well-formed and resolve four of the six concerns from that document's Appendix A.3 (placement, determinism, legacy migration, and binding extensibility). The items below are gaps and refinements, not objections to the decisions themselves.

### A.1 ADR-001 (Module Placement) — Endorsed, but the runtime/transport and sandbox impact are unspecified

The decision (desktop plugin UI + backend service boundary, UI must not own protocol semantics) is correct and directly resolves the placement concern.

Gaps to close before this is actionable:

- **Runtime and language are unnamed.** "Backend service/runtime" must say *what* it is. FEAGI binding selection depends on the `feagi-sensorimotor` coders (Rust) and the inference engine is Rust over ZMQ (`registration 5000`, `sensory 5555`, `motor 5556` per `feagi-inference-engine/src/main.rs`). State whether the Trainer service is a Tauri/Rust sidecar, a Python service, or co-located with the inference engine, and how it reaches FEAGI (ZMQ vs HTTP vs Tauri IPC).
- **App Store sandbox / XPC implications are unaddressed.** `feagi-desktop/docs/ARCHITECTURE.md` states sandboxed apps cannot spawn arbitrary subprocesses (the reason XPC services exist). A "backend service" that spawns processes or opens local sockets must be reconciled with that constraint and the "AITraining Service.xpc" slot. Add this to the decision or to a dependent ADR.
- **Control API transport is still open** (the design doc lists it as an open decision). ADR-001 should either fix it or explicitly defer it to a transport ADR.

### A.2 ADR-002 (Four-Axis Plugin Model) — Endorsed, but reward policy is a missing axis, and plugin trust is unaddressed

The four-axis model matches the design doc and the "binding selectors over native coders, no parallel codecs" principle is exactly right.

Refinements:

- **Reward policy should be first-class.** Per Appendix C.2 / D.2 of the design doc, the label/outcome-to-reward mapping is a per-task policy and should be a versioned axis (or explicit sub-contract), not folded silently into the encoder. ADR-002 currently omits it.
- **Plugin trust/security model is missing.** Plugins execute code and ingest external datasets. Given the design's "signed manifests" security goal, add a note (or ADR) on plugin provenance/signing and the trust boundary, especially under the App Store sandbox.
- Minor: state that every plugin-axis contract carries a `schema_version` and that the cross-axis compatibility matrix is itself versioned.

### A.3 ADR-003 (Determinism Semantics) — Strongly endorsed, with a GPU caveat and a FEAGI-core dependency

The two-level split (input-pipeline determinism guaranteed vs FEAGI-side reproducibility as a validated precondition) is the correct framing and fully resolves the determinism concern. Rejecting benchmark-mode runs that fail prechecks is the right governance stance.

Two additions:

- **GPU non-determinism must be explicit.** The inference engine has a `GpuConfig`. The ADR should state whether GPU-backed runs can ever be "benchmark-grade," and require recording a backend fingerprint (CPU/GPU, driver) — likely mandating CPU or a deterministic-GPU mode for published benchmarks.
- **The precheck depends on FEAGI exposing determinism controls.** "Fixed plasticity/reward policy configuration" and "deterministic runtime settings" assume FEAGI core can be put into a deterministic mode and can report it. If that control/introspection does not yet exist, this is a **FEAGI-core dependency** that should be called out as a prerequisite, not assumed.

### A.4 ADR-004 (Legacy Migration) — Endorsed, but "parity" and the bridge sunset must be made concrete

Time-boxed supersession with a temporary bridge is the right call and resolves the legacy-relationship concern.

Risks to harden:

- **Define "feature parity" explicitly.** M3 ("remove legacy paths after feature parity") is unfalsifiable without a parity checklist. The legacy stack supports CSV / image / video with a working correct/incorrect/fitness loop; enumerate which of those are in-parity targets and which (if any) are intentionally dropped.
- **The bridge must have a hard removal criterion and date/owner.** "Temporary compatibility bridge" is the classic trap that becomes permanent. Bind M3 to a concrete release window and owner (the approval checklist mentions this — make it a precondition of merging the bridge, not a later cleanup).

### A.5 Missing ADRs (recommended additions)

The current set (now including ADR-005 for UI and ADR-006 for the Rust/crate-reuse decision) still leaves several decisions from the design doc's "Open Decisions" and Appendix C/D unrecorded:

1. **ADR-007 Artifact storage backend** — filesystem vs object-store abstraction (design doc Open Decision 1).
2. **ADR-008 Contract serialization format** — JSON Schema vs protobuf+JSON, weighed against the Rust/RTOS migration goal and the coders' existing JSON properties (Open Decision 2).
3. **ADR-009 Reward-policy axis** — formalizing Appendix C.2 / D.2 (pluggable, versioned, part of the comparability key).
4. **ADR-010 Evaluation protocol versioning and comparability rules** — formalizing Appendix C.3 / D.3, including the rule that `compare_runs` rejects/flags cross-protocol-version and cross-reward-policy comparisons.
5. **ADR-011 Control API + Trainer↔FEAGI transport + `RunEvent` stream** — and its App Store sandbox/XPC implications (the unresolved part of ADR-001 and the transport dependency of ADR-005). This must also define the `check_dataset_compatibility` pre-registration capability query that serves upstream producers (Experience Capture preflight; design doc Section 6.2).
6. **ADR-013 Shared dataset-identity contracts and upstream-producer boundary** — ratifies the Option A vs B placement (ADR-006) and records the Experience Capture producer role for `dataset_asset_id`/`dataset_version` (ADR-012). Scoped to the consumer-side interface only; producer-side decisions stay in the Experience Capture decision log.

### A.6 Approval Checklist Additions

- [ ] ADR-001 names the service runtime/language, the FEAGI transport, and the sandbox/XPC reconciliation (note: ADR-006 fixes the language as Rust).
- [ ] ADR-002 records the reward-policy axis and a plugin trust/signing model.
- [ ] ADR-003 states GPU benchmark-grade policy and flags any FEAGI-core determinism-control dependency.
- [ ] ADR-004 includes a concrete parity checklist and a dated bridge-removal criterion with an owner.
- [ ] ADR-005 has a versioned `RunEvent` stream schema and a UI-contribution contract.
- [ ] ADR-006 crate-reuse set is pinned to a feagi-core workspace version and integration mode (embedded vs remote) is chosen per run-type.
- [ ] ADR-007..011 (or explicit deferral) exist for storage, serialization, reward policy, evaluation-protocol versioning, and transport/stream.
- [ ] ADR-011 includes the `check_dataset_compatibility` pre-registration query; ADR-013 (or explicit deferral) records the shared-contracts placement and the Experience Capture producer boundary.

---

## Appendix B. End-to-End Delivery Plan

This plan ties together all layers (contracts, backend service, plugin axes, UI, FEAGI-core dependencies, and legacy migration) so delivery is end-to-end, not layer-by-layer. It reconciles the design doc's capability phases (Section 9) with the migration milestones (ADR-004 M1–M3) and the UI contract (ADR-005).

### B.1 Layers (workstreams)

- **L0 Contracts/Schemas**: `DatasetManifest`, `IRSample` (with `OutputType`/`target`), `RunSpec` (pinned binding + `reward_policy` + `evaluation_protocol_version`), `EvaluationSpec`, `PredictionRecord`, `RunEvent` stream, plugin-axis descriptors.
- **L1 Backend Trainer Service**: orchestrator, dataset registry, validation/`validate_run` (binding resolution + pinning + compatibility chain), artifact store, Control API + `RunEvent` publisher.
- **L2 Plugin axes**: Adapter, Sampler, Encoder/Decoder (selectors over FEAGI coders), MetricPack, RewardPolicy — each with a conformance test.
- **L3 UI**: desktop plugin UI consuming Control API + `RunEvent` stream; plugin-contributed config/preview/result panels; no direct FEAGI protocol.
- **L4 FEAGI-core dependencies**: native coders/cortical areas (already present); a **pinned serialized connectome** for reproducibility (ADR-003 finding — no core change required); and any new coder for gap tasks (e.g. detection). Optional future core change: seedable development RNG.
- **L5 Migration**: supersede the legacy `feagi-react-core-pro` trainer per ADR-004.

### B.2 Delivery strategy: thin vertical slice first

Avoid building each layer horizontally. Land one **complete vertical slice** end-to-end before breadth — proposed slice: **tabular classification (IRIS)**, the simplest path that exercises every layer (Adapter -> scalar Encoder -> class Decoder -> classification MetricPack -> UI run + result). MNIST (image) is the second slice and validates the `ImageFrame` coder path.

### B.3 Phased plan (cross-layer)

**Phase 0 — Foundations and prerequisites (gate before build)**
- L0: author and review the core schemas (design doc Section 12 + `RunEvent`).
- L4: reproducibility approach is **resolved** — pin a serialized connectome (ADR-003 finding); no FEAGI-core change needed for the MVP. Confirm CPU-baseline benchmark policy (GPU fingerprinting deferred).
- Decisions: approve ADR-001..006; resolve ADR-011 (transport + stream) and ADR-007/008 (storage/serialization) at least minimally, since L1 depends on them.
- Exit: schemas frozen v1; transport chosen; Rust crate-reuse set pinned to a feagi-core workspace version.

**Phase 1 — Vertical slice (MVP, maps to design doc Phase 1 + ADR-004 M1)**
- L1: orchestrator + registry + `validate_run` + artifact store for one run lifecycle.
- L2: AdapterPlugin (tabular CSV), scalar Encoder + class Decoder selectors, classification MetricPack, baseline RewardPolicy (Pain/Pleasure).
- L3: minimal desktop plugin UI — import dataset, configure run, launch, observe `RunEvent` stream, view result; one plugin-contributed config panel to prove the UI-contribution contract.
- L5 (M1): the IRIS path runs entirely through service APIs (no direct-WS).
- **Scorecard (ADR-012)**: emit a `Scorecard` for the run's final connectome (pinned `connectome_hash`, local `dataset_asset_id`, `evaluation_protocol_version`) into the local artifact store; support local self-verification (re-run matches within tolerance).
- Exit: IRIS train/test run is reproducible, provenance-complete, produces a verifiable Scorecard, and is viewed end-to-end in the UI.

**Experiment integration (parallel to Phase 1–2, requires Gap-2 prerequisite)**
- Reuse `experiment_id`/`session_id`; capture the two-level learning curve (live online + pinned test-eval at checkpoints) and the per-run **final test score** as a Scorecard.
- Prerequisite (external, nrs-composer + desktop): run-scoped genome/connectome versioning (gap analysis Gap 2) so scorecards/checkpoints attach to specific snapshots, not an overwritten genome doc.

**Future (IDs reserved now, not built in MVP)**
- **Hosted dataset assets**: same `dataset_asset_id`/version resolve to a hosted asset instead of local.
- **Competitions / leaderboards** (closed/controlled): organizer-fixed `(dataset_version, evaluation_protocol_version, division)`, ranked Scorecards, reproducibility-verification integrity. Built on the Phase-1 Scorecard backbone; no competition-specific code in the MVP.

**Phase 2 — Breadth and legacy cutover (design doc Phase 1/2 + ADR-004 M2)**
- L2: add image-folder (MNIST) and COCO-like adapters; image Encoder path; stratified/curriculum SamplerPlugins.
- L3: Adapter-driven panels replace the fixed CSV/Image/Video tabs; remove hardcoded `127.0.0.1:9050`.
- L5 (M2): legacy UI routes invoke new service-backed run APIs; compatibility bridge active.
- Exit: MNIST + a COCO-like classification/gesture workflow run through the new stack; legacy UI is bridged.

**Phase 3 — Structured outputs and multimodal (design doc Phase 2 + Appendix B)**
- L2/L4: segmentation (`SegmentedImageFrame`) and 6DOF pose (`PoseEstimationData` + existing decoder) MetricPacks; text/video adapters.
- L0/L2: object detection gap work — new `bbox_set` `WrappedIOType` + detection decoder + mAP/IoU pack (its own milestone per design doc B.6).
- Exit: at least one segmentation and one pose benchmark run end-to-end.

**Phase 4 — Research-grade ops and legacy removal (design doc Phase 3 + ADR-004 M3)**
- L1/L3: run comparison enforcing `(evaluation_protocol_version, reward_policy_version)` comparability key; lineage/provenance visualization.
- L5 (M3): remove legacy direct-protocol paths after the parity checklist is met and the bridge-removal window closes.
- Exit: legacy trainer removed; published-benchmark governance in place.

### B.4 Critical path and risks

- **FEAGI-core determinism (L4)** is the top external dependency; if deterministic mode/introspection does not exist, Phase 0 must either schedule that core work or explicitly de-scope benchmark-grade reproducibility.
- **Transport/stream (ADR-011)** gates both L1 and L3; resolve in Phase 0.
- **Object detection** is research, not integration; keep it off the critical path (Phase 3 milestone) so it cannot block the MVP.
- **Bridge longevity (L5)**: enforce the dated removal criterion from the ADR-004 feedback so the temporary bridge does not become permanent.
