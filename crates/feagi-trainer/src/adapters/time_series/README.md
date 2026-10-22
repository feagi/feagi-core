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

An intensity code uses `normalize = none`: physical-unit samples are written as graded P.
`amplitude_offset` is then added to every encoded sample. It must be finite. Zero leaves
the scaled sample unchanged.

A population code uses `normalize = min_max_dataset`. Every sample on the configured
streams, across every loaded record, is scaled to `[0, 1]` with that recording range, then
placed on Z (`1.0` fires z 0, `0.0` fires the last bin, potential 1). A zero span is an
error. `amplitude_offset` must be 0 in this mode. Per-window and per-episode min-max stay
available for other `[0, 1]` paths; those ranges are not comparable across windows.

Snapshot presentation consumes the window (`feature_count` per selected stream) in one
sensory frame. Stream train uses the same window cut but sends one sample per burst
(`feature_count` must equal `pre+1+post`) onto Misc A with the class held on Misc B;
train does not collect the Misc OPU. Progress events carry `tick_index` and send
`window_values` on tick 0 so the desktop can draw the beat and a streaming playhead.
Stream infer emits one IRSample per episode (`normalize = none`, `min_max_per_episode`, or `min_max_dataset`)
and lists hold ends at `annotation + post`. At each hold end, infer scores a
motor frame when FEAGI publishes one and warns (does not fail) when it does not.
Stream presentations require exactly one `window.stream_id`.

`TimeSeriesPackageAdapter::preview` returns a page of raw (physical-unit) event
windows across all records (`frame_offset` + `frame_count`, plus `total_frames`,
`record_count`, and eligible-window `class_counts`).
It uses the same window and label policies as ingest and does not apply min-max
normalization or `amplitude_offset`. `value_min` and `value_max` are the minimum and
maximum physical samples on the configured streams across every loaded record.

`class_keep_percents` is an optional per-label map of integers `0..=100`. Empty
keeps every eligible window. Named labels keep `floor(count * percent / 100)`
windows in encounter order; omitted `class_labels` keep 100%. Unknown keys are
an error. Preview counts stay unfiltered. Stream infer rejects a non-empty map.
A positive percent that would keep zero windows is an error.

## Non-goals

- QRS detection or hand-crafted ECG features
- Hardcoded AAMI maps
- InfluxDB / row-store ingest
- Playing a `.dat` file as a live monitor outside Trainer (that is Capture). Stream infer is
  the Trainer-side continuous-record path for the same analog corpus.
