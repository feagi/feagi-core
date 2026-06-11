# FEAGI Trainer — Training Paradigms

Status: Proposed
Date: 2026-06-09
Owners: FEAGI Trainer Architecture Working Group
Related: `docs/FEAGI_TRAINER_ARCHITECTURE_AND_DESIGN.md`, `docs/FEAGI_TRAINER_ADR_SET.md`, `docs/EXPERIENCE_TRAINER_E2E_IMPLEMENTATION_PLAN.md`, `docs/EXPERIENCE_CAPTURE_ARCHITECTURE_AND_DESIGN.md`, crate `feagi-core/crates/feagi-trainer`

This document enumerates the training paradigms `feagi-trainer` accounts for, the mechanism each uses, and its implementation status. It is descriptive of the agreed design; paradigm scope changes must be discussed and reflected here and in the architecture/design doc before implementation.

---

## 1. Framing — the Trainer shapes signals; FEAGI does the learning

FEAGI is **not** a gradient learner. Learning physically happens *inside* the engine via plasticity (STDP) driven by neural co-activation and FEAGI's native affect channels (**Pain / Pleasure / Fear / Hope**) — see `FEAGI_TRAINER_ARCHITECTURE_AND_DESIGN.md` A.3 #1 and Appendix B.3.

Consequently, a "training paradigm" in `feagi-trainer` is defined by **how a supervisory signal is shaped and which channel it is delivered into** — not by which optimizer runs. The Trainer orchestrates signals and scores outcomes; it never implements a learning rule.

There are exactly three delivery channels into FEAGI, and every paradigm is a combination of them:

| Channel | Where in the loop | Carries | Trainer axis |
|---|---|---|---|
| Sensory (IPU) | before `step` | the stimulus / observation | `EncoderPlugin` |
| Affect / reward (Pain/Pleasure/Fear/Hope) | after `collect_motor` | outcome valuation | `RewardPolicy` (first-class axis) |
| Teaching / target-motor | with sensory, before `step` | the demonstrated correct action | reserved (`FeagiRuntime::submit_target_motor`) |

The reward axis is anchored in FEAGI's native affect areas (Appendix B.3); the Trainer must not invent a side-channel reward mechanism. Reward-policy version is part of the run comparability key (ADR / Appendix D.2).

### 1.1 The Trainer delivers these channels *as a FEAGI agent*

The three channels above are delivered over `feagi-agent`. This means the Trainer **is itself a FEAGI agent**, and how it co-exists with an embodiment controller is the central topology decision (**ADR-014**):

- **Non-embodied data (datasets):** there is no controller, so the Trainer is the **sole agent** — it drives the sensory input *and* the reward/pain.
- **Embodied tasks:** the embodiment controller is already a FEAGI agent owning the robot's real sensory/motor + sim physics. The Trainer runs as a **parallel co-agent on disjoint cortical I/O**, owning only the training-signal channels (reward/teaching/goal) + readouts for scoring. It **never drives sim physics**.

Capture/replay stays embodiment-agnostic: capture at the **cortical (FEAGI-native) boundary**; embodiment-native traces are an opt-in sidecar; the Trainer never speaks a controller's native command language (**ADR-015**).

---

## 2. Paradigms

### 2.1 Reward & punishment (reinforcement-style)
The first-class, versioned `RewardPolicy` axis maps an outcome onto the native affect channels. This is the backbone of every supervised and reinforcement run. Two flavors:

- **Supervised-correctness reward** — prediction-vs-label drives Pleasure/Pain (e.g. the IRIS slice via `PainPleasureReward`). **Implemented.**
- **Environmental / episodic reward** — the outcome of an embodied rollout drives affect, with success evidence from a telemetry predicate or goal-distance signal (ADR-014). Delivered by the Trainer **co-agent** in parallel with the controller. When reward is intrinsic to the genome (e.g. the pendulum's R-STDP `balance_homeostatic` personality), this axis is a **no-op / observe-only** and the Trainer scores rather than shapes. This flavor cannot be expressed by the current `RewardPolicy::reward(prediction, target)` signature (it has no per-step target); it requires the co-agent seam (Section 3).

### 2.2 Supervised associative learning
Present the input *and* a correctness / expected signal during a train phase so FEAGI associates stimulus → response. Mechanically this is delivered *through* the reward channel (the policy emits the correctness signal into the affect areas during the train phase — Appendix B.3). In FEAGI terms "supervised" is a special case of reward-shaping, not a separate optimizer. **Implemented** for classification (IRIS).

### 2.3 Imitation / behavior cloning (teaching / supervised forcing)
Inject the *demonstrated action* as expected motor output before the burst, teaching FEAGI to reproduce it. This uses the **reserved** teaching / target-motor channel (`FeagiRuntime::submit_target_motor`, default `Unsupported`). The seam exists (plan Phase 1b) so the loop is not reopened later; the mode itself is **scheduled for Phase 5** (VLA slice — learning from demonstrations alongside reward).

### 2.4 Closed-loop reinforcement / embodied control
Roll the brain out in an environment, deliver per-step / per-episode reward, and score episodic success (e.g. balance duration, cumulative reward, success rate). This is the embodied/control metric family (design Section 5.8). The **topology is decided** (ADR-014): the Trainer is a **parallel co-agent** that injects reward/teaching/goal signals and scores, while the controller owns the robot's sensory/motor + sim physics. The **episodic-control metric pack + `EpisodeTrajectory` are built and tested**; the live co-agent execution path is sequenced after the dataset path (plan Phase 1d, re-scoped). Embodied Scorecards mean **closed-loop task success**, not offline prediction accuracy (plan Section 5.6).

### 2.5 Unsupervised / Hebbian associative exposure
Present sensory input only — no reward and no target — and let plasticity associate co-active patterns. This is **structurally already possible** (the executor records unlabeled samples without scoring or rewarding them) but is **not a first-class, named scenario or metric pack yet**. A common **goal-driven** variant (e.g. quadruped: "minimize deviation from a target IMU stability signal") is implemented as **reward shaping** — the goal-distance becomes the affect signal (§2.1 / ADR-014), so it reduces to the reward axis rather than a new mechanism. A genuinely self-supervised/predictive objective remains a deferred, open decision.

---

## 3. Cross-cutting seam note

Paradigms 2.1 (supervised flavor) and 2.2 share the `RewardPolicy::reward(prediction, target)` seam and the static-sample executor `run_rollout`. Paradigms 2.3, 2.4, and 2.5 each break that signature:

- 2.4 has **no per-step target** and **environment-derived reward**;
- 2.3 needs the **teaching / target-motor** channel;
- 2.5 has **no reward and no target** at all.

The **co-agent seam (ADR-014)** resolves this generally: because the Trainer injects training signals on disjoint cortical I/O, hosting environment-reward (2.4), teaching (2.3), and exposure/goal (2.5) is a matter of *which channel the co-agent drives*, not a new loop. Until the live co-agent path is built, only 2.1 (supervised) and 2.2 are exercised end-to-end.

---

## 4. Status summary

| Paradigm | Primary channel(s) | Status | Milestone |
|---|---|---|---|
| Reward & punishment — supervised-correctness | affect | Implemented | M1 (IRIS) |
| Supervised associative classification | sensory + affect | Implemented | M1 (IRIS) |
| Reward & punishment — environmental/episodic | affect (co-agent) | Metric pack built; live co-agent path pending | Phase 1d (re-scoped) |
| Closed-loop embodied control (RL-style) | sensory + affect (co-agent) | Topology decided (ADR-014); metric pack built; live path pending | Phase 1d (re-scoped) |
| Imitation / behavior cloning | teaching / target-motor | Reserved seam (Phase 1b) | Phase 5 |
| Unsupervised / Hebbian exposure | sensory only | Structurally possible, not first-class | Open decision |
| Goal-driven (reward-shaped) | affect (co-agent) | Reduces to §2.1 reward shaping | Phase 1d (re-scoped) |

---

## 5. Explicit non-goals

- **No gradient descent / backprop and no Trainer-side learning rule** — plasticity / STDP lives in the FEAGI engine.
- **No Trainer-invented reward mechanism** — reward must target FEAGI's native affect channels (Appendix B.3).
- **No data capture or labeling** — that is Experience Capture's responsibility (ADR-001 boundary); the Trainer consumes datasets and produces verifiable Scorecards.

---

## 6. Worked scenarios (how the paradigms compose)

These worked examples show the design covers diverse training needs by varying only **which signals the Trainer co-agent injects** and **the data source** — not the engine (ADR-014/ADR-015).

| # | Scenario | Paradigm(s) | Agent topology | Trainer agent injects | Reward / success evidence | Data source |
|---|---|---|---|---|---|---|
| 1 | Robot arm — pickup from manual demos | Imitation (2.3) + reward (2.1) | Co-agent (controller runs the arm) | demonstration/teaching + reward/pain | telemetry predicate ("object lifted") | Experience Capture episodes (+ live sim) |
| 2 | Robot arm — coordinates → pickup | Supervised assoc. (2.2) + reward (2.1) | Co-agent | object-coordinate input + reward/pain | experience labels | Experience collection |
| 3 | Cancer-cell anomaly detection | Supervised classification (2.2 + 2.1) | **Sole agent** (no embodiment) | dataset sample input + reward/pain | experience labels (valid vs normal) | dataset |
| 4a | Quadruped walk — IMU stability goal | Goal-driven, reward-shaped (2.5→2.1) | Co-agent | goal (target IMU) signal | goal-distance to ideal IMU | captured IMU stability |
| 4b | Quadruped walk — gait demonstration | Imitation (2.3) + goal (2.5→2.1) | Co-agent | gait demonstration + ideal-IMU goal | goal-distance + demo alignment | Experience Capture |

Common thread: the controller (when present) owns the robot's sensory/motor + physics; the Trainer co-agent adds reward/teaching/goal on disjoint cortical I/O and scores the outcome. For scenario 3 there is no controller, so the Trainer is the sole agent — this is the **dataset path built first**.
