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

- `contracts`: the public, versioned data contracts that form the stable seam between this
  crate and its consumers. v1 types: `DatasetManifest`, `IRSample`, `RunSpec`, `Scorecard`.

Engine wiring (adapters, samplers, encoder/decoder binding selectors over
`feagi-sensorimotor` coders, metric packs, deterministic run execution) lands next.

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
  contracts/
    common.rs             shared IDs, hashes, plugin refs, taxonomies
    dataset_manifest.rs   DatasetManifest
    ir_sample.rs          IRSample + typed target / output-type taxonomy
    run_spec.rs           RunSpec (pinned binding, reward policy, eval protocol)
    scorecard.rs          Scorecard (status + visibility)
tests/
  contracts_roundtrip.rs  serde round-trip integration tests
```
