# Genome Schema Versioning

**Status:** Approved (design only; no implementation yet)
**Owners:** feagi-core / feagi-evolutionary
**Last updated:** 2026-04-24

## Goal

Replace the monolithic genome migrator with a stepwise chain of small,
isolated `vN → vN+1` migrators paired with per-version validators. Make
`genome_schema_version` load-bearing so dispatch is deterministic
instead of shape-sniffing.

## Scope

- Applies to `feagi-evolutionary::genome::*`.
- Out of scope: representation coercion (flat ↔ hierarchical), runtime
  genome behavior, NPU/burst-engine integration, amalgamation.

## Decisions (locked)

| # | Decision | Rationale |
|---|---|---|
| 1 | Add `genome_schema_version: u32` to every genome, decoupled from the human-facing `version` string. | Total order, no gaps, idiomatic Rust + JSON. |
| 2 | Migrators operate on `serde_json::Value`, not `RuntimeGenome`. | Old shapes cannot deserialize into the current typed form. |
| 3 | One file per `vN → vN+1` step (`v2_to_v3.rs`, `v3_to_v4.rs`, …). | Modularity, isolated tests, no monolith. |
| 4 | Pipeline order: deserialize → normalize representation → detect schema version → migrate(N→N+1)\* → validate@latest → parse to `RuntimeGenome`. | Separation of concerns. |
| 5 | Forward-only. No down-migrators. | Doubles surface area for no current need. |
| 6 | `feagi-evolutionary` retains the **full** migrator chain forever. `nrs-composer` retains only the last **K=5** validators as first-class policy gates. | Open-source crate is the historical record; deployed service is bounded. |
| 7 | Per-version validators run between hops as **advisory**; only the latest validator is **blocking**. | Catch corruption early without bricking historical genomes. |
| 8 | `validate-and-repair` surfaces `from_version`, `to_version`, `migrators_applied` in addition to existing fields. | Eliminates the silent auto-save failure path. |

## Composer integration (option C)

`nrs-composer`:

- Imports the **full migrator chain** from `feagi-evolutionary` as a
  library entry point. Migration is bounded compute and deterministic.
- Imports only the **last K=5 validators** as policy gates.
- Genomes arriving at a schema version older than `latest − K + 1` are
  migrated to the lower bound of the validator window first; any
  blocking errors at that lower bound surface as HTTP 4xx with a
  structured `migrator_diagnostics` payload.
- This is the `validate-and-repair` integration point referenced in
  decision #8.

## Types

```rust
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct GenomeSchemaVersion(pub u32);

// The integer space starts at 2. No v1 was ever persisted to the
// production database or the offline `g0/` corpus, so reserving a v1
// slot would be fiction. The chain registry is contiguous starting at
// MIN_SCHEMA_VERSION.
pub const MIN_SCHEMA_VERSION: GenomeSchemaVersion = GenomeSchemaVersion(2);
pub const CURRENT_SCHEMA_VERSION: GenomeSchemaVersion = GenomeSchemaVersion(3);

pub trait Migrator: Send + Sync {
    fn from_version(&self) -> GenomeSchemaVersion;
    fn to_version(&self) -> GenomeSchemaVersion;
    fn migrate(
        &self,
        genome: &mut serde_json::Value,
    ) -> Result<MigrationStepDiagnostics, MigrationError>;
}

pub trait Validator: Send + Sync {
    fn schema_version(&self) -> GenomeSchemaVersion;
    fn validate(&self, genome: &serde_json::Value) -> ValidationReport;
}
```

## Chain runner contract

- Walks `src → tgt` monotonically. Step `to_version` of each migrator
  must equal `from_version` of the next; the runner refuses to start
  otherwise.
- Between hops, runs `Validator(vN+1)` as **advisory**; collects
  diagnostics.
- Final hop: runs `Validator(vLatest)` as **blocking**.
- Emits a `ChainResult` (see Diagnostics).
- Fails fast on `MigrationError`. Validation issues at intermediate
  steps never abort the chain.

## Module layout

```
feagi-evolutionary/genome/
  schema/
    version.rs        // GenomeSchemaVersion + MIN/CURRENT constants
    detector.rs       // detect_schema_version(&Value): reads the
                      // integer field if present; otherwise back-fills
                      // from the legacy `version` string per the table
                      // below. Rejects everything else. No shape
                      // sniffing.
  migration/
    mod.rs            // Migrator trait + chain registry + ChainResult
    chain.rs          // pipeline runner
    v2_to_v3.rs       // one file per step; initial chain has exactly
                      // this one step
    ...
  validators/
    mod.rs            // Validator trait + ValidationReport
    v2.rs             // one file per version
    v3.rs             // blocking validator at latest
    ...
  loader.rs           // orchestrator only — no migration logic, no
                      // validation logic, no auto-fix logic
```

### Legacy `version` string back-fill

The detector populates `genome_schema_version` from the legacy
human-facing `version` string only at the deserialize boundary, exactly
once per genome:

| Legacy `version` string | Assigned `genome_schema_version` |
|---|---|
| `"2.0"` | `2` |
| `"2.1"` | `2` |
| `"3.0"` | `3` |
| anything else (including missing) | reject with structured error |

`"2.1"` is included because it is carried by shipped embedded fixtures
(`essential_genome.json`, `vision_genome.json`). It is structurally
identical to `"2.0"`; both pass through V2ToV3Migrator unchanged.

This table is closed. New schema versions are introduced by writing the
integer field directly; they do not get a corresponding legacy string.
Adding a new entry to the table (e.g. if another minor variant is found
in the wild) requires an explicit code change and corresponding test in
`schema/detector.rs`.

## Diagnostics

```rust
pub struct MigrationStepDiagnostics {
    pub from_version: GenomeSchemaVersion,
    pub to_version: GenomeSchemaVersion,
    pub transformations: Vec<String>,
}

pub struct ChainResult {
    pub from_version: GenomeSchemaVersion,
    pub to_version: GenomeSchemaVersion,
    pub migrators_applied: Vec<&'static str>,
    pub per_step_diagnostics: Vec<MigrationStepDiagnostics>,
    pub advisory_warnings: Vec<String>,
    pub blocking_errors: Vec<String>,
}
```

`validate-and-repair` maps `ChainResult` directly into its JSON
response.

## Test strategy

- **Per migrator:** golden-file before/after fixtures in
  `tests/fixtures/genome/vN_to_vN+1/{before.json, after.json}`. Test
  asserts byte-equality after canonical JSON normalization.
- **Per validator:** positive + negative property tests at each schema
  version.
- **Chain-level:** end-to-end "vMin → vLatest" round-trip on every
  committed fixture, on every PR.
- No mocks. All tests operate on real `serde_json::Value`.

## Out of scope (explicit)

- Reverse migrations.
- Auto-detection beyond the `genome_schema_version` field, except for
  the closed legacy-string back-fill table in `schema/detector.rs`.
  Shape sniffing is forbidden.
- Cross-genome operations (e.g. amalgamation).
- Migrating embeddings, telemetry payloads, or any non-genome data.

## Implementation order

Each numbered item is a separate PR. The real-world version space at
design time is the closed back-fill table (`"2.0"`, `"2.1"`, and `"3.0"`
in the legacy string field — see [Legacy back-fill](#legacy-version-string-back-fill)),
which keeps the initial chain trivially small.

1. Land `GenomeSchemaVersion(u32)`, `MIN_SCHEMA_VERSION = 2`,
   `CURRENT_SCHEMA_VERSION = 3`, and the `genome_schema_version` field.
   Implement `schema/detector.rs` per the legacy back-fill table.
   Plumb the field through serialize/deserialize. Migrate the database
   in place with the two `updateMany` statements in the back-fill
   section. No behavior change beyond the new field.
2. Land traits + chain runner with a single no-op `Migrator(v3 → v3)`
   placeholder and a single `Validator(v3)`. Wire `loader.rs` to use
   the chain runner. Genomes back-filled to `2` are intentionally
   rejected by the chain at this step — the runner refuses to start
   when the registry has no `v2 → v3` migrator yet. This is gated
   behind a feature flag until step 4 lands.
3. Relocate the existing `auto_fix_genome` logic into a `v3`
   normalizer migrator (`Migrator(v3 → v3)` non-noop). The validator
   stops mutating; mutation belongs in migrators only.
4. Replace the noop `v2 → v3` slot with the real migrator: the
   existing monolithic `migrate_genome` body, repackaged as a single
   step. Drop the feature flag. End-to-end fixtures from `g0/` (all
   `"2.0"`) round-trip cleanly.
5. Update `nrs-composer` `validate-and-repair` to surface
   `from_version`, `to_version`, `migrators_applied` per decision #8.
6. Update `feagi-desktop` to display these fields and to treat
   `migrator_diagnostics` as user-visible information instead of a
   silent `console.warn`.

Future schema bumps (v4, v5, …) follow the procedure documented in the
module-level README (`feagi-evolutionary/src/genome/README.md`). One PR
per `vN → vN+1` step. The existing `v2 → v3` migrator is **not** to be
retroactively split into smaller hops — that is archaeology with no
payoff.

## References

- Current monolithic migrator: `feagi-evolutionary/src/genome/migrator.rs`
- Current validator: `feagi-evolutionary/src/validator.rs`
- Composer integration: `feagi-desktop/src-tauri/src/commands/genomes.rs::validate_and_repair_via_composer`
- Auto-save circuit breaker: `feagi-desktop/src-tauri/src/commands/auto_save_circuit.rs`
- Module-level rules: `feagi-evolutionary/src/genome/README.md`
