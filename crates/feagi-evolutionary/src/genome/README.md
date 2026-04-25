# Genome Versioning Rules

Authoritative rules for evolving the genome schema. See the system-level
design at [`docs/GENOME_SCHEMA_VERSIONING.md`](../../../../docs/GENOME_SCHEMA_VERSIONING.md)
for the architecture, decisions, and rationale.

## The version field

Every genome MUST carry an integer field `genome_schema_version: u32`.
The current latest version is defined by the constant
`schema::version::CURRENT_SCHEMA_VERSION`. The lowest recognized
version is `schema::version::MIN_SCHEMA_VERSION`.

The integer space starts at **2**. There is no `v1` and there will
never be one — the project never persisted a genome at that integer in
the production database or the offline `g0/` corpus. Reserving a v1
slot would be fiction and would weaken the "registry is contiguous"
invariant for no benefit.

The legacy `version` string field is preserved as a human-readable label
only. It MUST NOT be used to drive code logic. Anything that branches
on `genome.version` is a bug. The detector at `schema/detector.rs`
back-fills the integer field from the legacy string at the deserialize
boundary using a closed table — `"2.0" → 2`, `"2.1" → 2`, `"3.0" → 3`, anything
else rejected. New schema versions do not get a corresponding legacy
string; they live as integers only.

## When to bump `CURRENT_SCHEMA_VERSION`

Bump the version if and only if the change satisfies any of:

- A new **required** field is added.
- A field is removed or renamed.
- The semantics of an existing field change (allowed values, units,
  encoding, default behavior).
- The shape of a nested container changes (e.g. array → object,
  flat → nested).

You MUST NOT bump the version for:

- Adding an **optional** field with a forward-compatible default.
- Internal refactors that don't affect on-disk shape.
- Documentation, comment, or naming-only changes.
- Performance improvements that preserve semantics byte-for-byte.

If unsure, propose the bump in the PR and let review decide. False
positives are cheap; false negatives corrupt user data.

## How to add a new schema version

1. Increment `CURRENT_SCHEMA_VERSION` by 1 in `schema/version.rs`.
2. Create `migration/vN_to_vN+1.rs` implementing `Migrator`. The
   migrator MUST satisfy the [invariants](#invariants-every-migrator-must-preserve)
   below.
3. Create `validators/vN+1.rs` implementing `Validator` for the new
   shape.
4. Add golden-file fixtures under
   `tests/fixtures/genome/vN_to_vN+1/{before.json, after.json}`.
5. Add a unit test that runs the migrator on `before.json` and asserts
   exact equality with `after.json` after canonical JSON normalization.
6. Register the migrator in `migration/mod.rs`. The registry MUST be a
   contiguous sequence — no gaps, no duplicates. The chain runner
   refuses to start if it isn't.
7. Update `nrs-composer` to bundle the new validator if it falls inside
   the K=5 retention window. This is a separate PR in the composer
   repo.

## Invariants every migrator must preserve

- **Determinism.** Same input → same output, byte-for-byte after
  canonical JSON normalization. No timestamps, no UUIDs, no
  iteration-order leaks.
- **Idempotence on input.** Running a migrator on a genome already at
  its `from_version` and applying it twice is allowed only if the
  second application is a no-op (other than diagnostics).
- **Bounded compute.** No allocation or work that scales worse than
  O(n) in the size of the input genome.
- **No side channels.** A migrator may not log user-identifying data,
  emit telemetry, perform I/O, read environment variables, or touch any
  clock.
- **JSON only.** A migrator MUST NOT take or produce `RuntimeGenome`.
  It operates on `serde_json::Value`.
- **Diagnostics over silence.** Every transformation produces at least
  one entry in `MigrationStepDiagnostics.transformations`. A migrator
  that runs and produces zero diagnostics is a bug.

## Forward-only

Down-migration is not supported and not planned. Cloud-stored genomes
are migrated in place exactly once and never written back at an older
version. If rollback is needed, restore from a versioned backup.

## Retention

- **`feagi-evolutionary` (this crate):** retains the **full** chain
  forever. Old migrators are immutable historical artifacts and MUST
  NOT be edited except for compilation-blocking dependency updates,
  which require explicit review and a regression test on every fixture
  newer than the change.
- **`nrs-composer` (deployed service):** retains only the last **K=5**
  validators as first-class policy gates. Older validators are
  advisory only. See `docs/GENOME_SCHEMA_VERSIONING.md` for the
  composer integration contract (option C).

## Validation placement

- Validators run **between hops** as advisory. Their reports are
  collected into `ChainResult.advisory_warnings` but never abort the
  chain.
- Only `Validator(vLatest)` is **blocking**. Its errors land in
  `ChainResult.blocking_errors` and cause the loader to refuse the
  genome.
- A migrator MUST NOT perform validation. Validation lives in
  `Validator`. If a migrator finds a structural problem it cannot
  reconcile, it returns `MigrationError`, not a validator-style
  `ValidationReport`.

## Anti-patterns (do not do)

- Branching on `genome.version` (the string) instead of
  `genome_schema_version` (the integer).
- Sniffing data shape **anywhere**. The chain runner dispatches on the
  integer `genome_schema_version`; migrators run unconditionally on
  their declared input. The detector reads the integer field or
  back-fills from the closed legacy-string table — it does not
  inspect data shape. If you ever feel the need to shape-sniff, you
  are looking at unversioned data; reject it at the deserialize
  boundary instead of guessing.
- Editing a previously released migrator to handle a corner case
  discovered later. Add a new migrator step instead.
- Performing validation inside a migrator.
- Producing `RuntimeGenome` from a migrator.
- Returning silent success when a transformation was applied.
- Bumping `CURRENT_SCHEMA_VERSION` without writing the corresponding
  migrator and validator in the same PR.
