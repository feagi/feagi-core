# Time-series adapter

Ingests annotated multi-channel analog recordings and emits event-window classification
`IRSample`s for FEAGI Trainer.

This is a **format family**, not a dataset brand. MIT-BIH is one WFDB archive. PTB-XL or a
local analog package uses the same adapter with a different `TimeSeriesPackageConfig`.

## Sources

- `source_kind: wfdb` — directory of `.hea` / `.dat` / annotation files. Formats `16` and
  `212` only. Every header signal must appear exactly once in `channel_map`. MIT
  annotation AUX length is taken from the AUX word (`getann`), not from the first
  payload byte.
- `source_kind: package` — canonical analog package (`manifest.json`, `stream_schema.json`,
  `episodes.json`, `events.jsonl`, `streams/<episode>/<stream_id>.f32le`).

## Task view

Windowing, class labels, analog scaling, and edge/unknown-label policies are
**explicit config**. The adapter never infers them and never silently drops events unless
the chosen policy is `exclude`.

ECG trainer runs use `normalize = none`: physical-unit samples are written as graded P.
Min-max modes remain available for population Z-bin encoders that require `[0, 1]`.

Snapshot presentation consumes the window (`feature_count` per selected stream) in one
sensory frame. Stream train uses the same window cut but sends one sample per burst
(`feature_count` must equal `pre+1+post`) onto Misc A with the class held on Misc B;
train does not collect the Misc OPU.
Stream infer emits one IRSample per episode (`normalize = none` or `min_max_per_episode`)
and lists hold ends at `annotation + post`. At each hold end, infer scores a
motor frame when FEAGI publishes one and warns (does not fail) when it does not.
Stream presentations require exactly one `window.stream_id`.

`TimeSeriesPackageAdapter::preview` returns a page of raw (physical-unit) event
windows across all records (`frame_offset` + `frame_count`, plus `total_frames`,
`record_count`, and eligible-window `class_counts`).
It uses the same window and label policies as ingest and does not apply min-max
normalization.

## Non-goals

- QRS detection or hand-crafted ECG features
- Hardcoded AAMI maps
- InfluxDB / row-store ingest
- Playing a `.dat` file as a live monitor outside Trainer (that is Capture). Stream infer is
  the Trainer-side continuous-record path for the same analog corpus.
