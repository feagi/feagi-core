# Genome Change Ledger

Records every successful structural change to the live genome: what changed, who
changed it, and when. Changes can be replayed on another FEAGI instance.

## Design

- `RecordingGenomeService` and `RecordingConnectomeService` wrap the service
  implementations. They pass every call through and record only successful mutations,
  so the implementations stay unchanged. Composite API operations such as clone
  become several entries that share one `group_id`.
- `ChangeLedger` assigns a process-local `sequence` (polling cursor) and a global
  `change_id` (UUID v7, the idempotency key). It keeps the most recent entries in
  memory and appends each replayable entry, without `sequence`, to the live genome's
  `change_history`. A saved genome therefore carries its own history. Signatures do
  not cover `change_history`.
- `ChangeContext` (task-local) carries the caller identity and the request group. The
  HTTP layer sets it from `X-FEAGI-Agent-Id`. Work spawned onto another task is
  recorded without an agent id.
- `apply_change` replays a peer's entry through the recording services under a replay
  context. The entry keeps its original id, group, timestamp and agent, and is marked
  `origin: replayed` so relays can skip it and avoid echoing it back. An id that is
  already present is a no-op. A change whose target is missing fails and is not
  recorded.
- `GenomeLoaded`, `ConnectomeReset` and `ConnectomeImported` are markers. They are not
  replayable and are not persisted.

## API (feagi-api)

- `GET /v1/genome/changes?since=<seq>&limit=<n>` returns `{changes, latest_sequence,
  oldest_available_sequence, gap, has_more}`. When `gap` is true, entries after the
  cursor were evicted and the caller must resynchronize from the full genome.
- `POST /v1/genome/changes/apply` accepts a `GenomeChange` from a peer as-is and
  returns `{status: applied, sequence}` or `{status: already_applied}`.
- `health_check.genome_change_sequence` advances on every recorded change.

## Configuration (`[genome]`)

- `change_ledger_session_capacity`: entries held in memory for polling.
- `change_history_max_entries`: entries persisted in the genome (oldest dropped first).
- `change_agent_id_max_length`: longest accepted agent id.

## Known limits

- IO areas created automatically during agent registration are recorded without an
  agent id.
- `load_genome` develops into the process-wide `ConnectomeManager` (pre-existing), so
  the two-instance tests seed their genomes directly.
