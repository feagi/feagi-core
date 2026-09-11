# feagi-dataset-contracts

Shared, dependency-light primitives for the FEAGI dataset/experience interchange contracts
(Apache-2.0).

## Purpose

This crate is the single source of truth for the small building blocks that the larger
contract aggregates compose — typed identifier newtypes (`DatasetAssetId`,
`DatasetVersionId`, `ContentHash`, `ConnectomeHash`, `PluginId`, `SplitId`, ...), the
`PluginRef` reference, the `QuantizationFingerprint`, and the cross-cutting taxonomy enums
(`Modality`, `Split`, `OutputType`, `BackendKind`, `MetadataValue` / `MetadataMap`).

It is extracted (Phase 1a of `docs/EXPERIENCE_TRAINER_E2E_IMPLEMENTATION_PLAN.md`, decision
Option B in Section 3) so both consumers can share one definition without coupling:

- `feagi-trainer` — the train / evaluate / benchmark engine, and
- `feagi-experience-capture` — the capture / packager (planned).

Neither consumer pulls the other's engine through this crate.

## Design rationale

- **Lean by requirement.** The Nano deployment profile depends on the capture side, so this
  crate stays serde-only: no engine, no I/O, no runtime. Its only dependencies are `serde`
  and `serde_json`.
- **Deterministic on the wire.** Identifier newtypes are `#[serde(transparent)]` (bare
  strings); taxonomy enums are `snake_case`; `MetadataValue` is untagged; metadata maps use
  `BTreeMap` for stable key ordering.
- **No implicit fallbacks.** Callers set every field explicitly.
- **Rust/RTOS-migration friendly.** Statically typed with minimal dynamic behavior.

## Dependencies

- `serde` (derive)
- `serde_json` (for the opaque `QuantizationFingerprint.details` value)

## Tests

`tests/primitives_roundtrip.rs` pins the serialized wire format of the primitives. Run with:

```bash
cargo test -p feagi-dataset-contracts
```
