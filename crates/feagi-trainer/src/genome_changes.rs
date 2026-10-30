//! Genome changes during a run, written to the run's event log.
//!
//! FEAGI keeps a change ledger of every structural genome change. [`GenomeChangeSink`] wraps a
//! [`RunDataSink`]: when a row of the watched table (the attempts table) arrives, it asks a
//! [`GenomeChangeSource`] for the changes recorded since the last look and writes one
//! `genome_change` event per change, tagged with the episode (attempt) number of that row. A
//! change is therefore tied to the attempt during which, or just before which, it was seen.
//!
//! The source is a trait so the host supplies the transport (this crate has no REST client).
//! A source that fails is logged once in [`GenomeChangeSink::failure`] and then ignored: the
//! run never fails over this.

use crate::run_data::{EventTime, RunDataSink, RunDataValue, TableSpec};
use serde_json::Value;

/// Event kind of one genome change.
pub const GENOME_CHANGE_KIND: &str = "genome_change";
/// Event kind written once when entries were evicted before they could be read.
pub const GENOME_CHANGE_GAP_KIND: &str = "genome_change_gap";
/// Longest label, in characters.
pub const LABEL_MAX_CHARS: usize = 120;
const PAGE_LIMIT: usize = 500;

/// One page of the ledger, as `GET /v1/genome/changes` returns it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChangesPage {
    /// Changes after the cursor, oldest first (raw ledger entries).
    pub changes: Vec<Value>,
    /// Highest sequence recorded so far.
    pub latest_sequence: u64,
    /// Entries after the cursor were evicted.
    pub gap: bool,
    /// More entries exist after the last one returned.
    pub has_more: bool,
}

impl ChangesPage {
    /// Read a page from the endpoint's JSON.
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let latest_sequence = value
            .get("latest_sequence")
            .and_then(Value::as_u64)
            .ok_or("changes reply has no latest_sequence")?;
        Ok(Self {
            changes: value
                .get("changes")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            latest_sequence,
            gap: value.get("gap").and_then(Value::as_bool).unwrap_or(false),
            has_more: value
                .get("has_more")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
    }
}

/// Where the changes come from.
pub trait GenomeChangeSource {
    /// At most `limit` changes with a sequence greater than `since`.
    fn page(&mut self, since: u64, limit: usize) -> Result<ChangesPage, String>;

    /// A readable name for a target (`kind` is the ledger's target kind, for example
    /// `cortical_area`), or `None` to show the id.
    fn display_name(&mut self, _kind: &str, _id: &str) -> Option<String> {
        None
    }
}

/// A sink that adds `genome_change` events after each row of one table.
pub struct GenomeChangeSink<'a> {
    inner: &'a mut dyn RunDataSink,
    source: &'a mut dyn GenomeChangeSource,
    table: String,
    column: String,
    column_index: Option<usize>,
    cursor: Option<u64>,
    failure: Option<String>,
}

impl<'a> GenomeChangeSink<'a> {
    /// Wrap `inner`. The cursor is read now (run start). `table` is the table whose rows end an
    /// attempt, and `column` its attempt-number column.
    pub fn new(
        inner: &'a mut dyn RunDataSink,
        source: &'a mut dyn GenomeChangeSource,
        table: &str,
        column: &str,
    ) -> Self {
        let mut sink = Self {
            inner,
            source,
            table: table.to_string(),
            column: column.to_string(),
            column_index: None,
            cursor: None,
            failure: None,
        };
        match sink.source.page(0, 1) {
            Ok(page) => sink.cursor = Some(page.latest_sequence),
            Err(error) => sink.failure = Some(error),
        }
        sink
    }

    /// Why change tracking stopped, when it did. Logged once by the host.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    fn collect(&mut self, episode: Option<u64>) {
        let Some(mut since) = self.cursor else {
            return;
        };
        let at = EventTime {
            episode,
            ..EventTime::default()
        };
        loop {
            let page = match self.source.page(since, PAGE_LIMIT) {
                Ok(page) => page,
                Err(error) => {
                    self.failure = Some(error);
                    self.cursor = None;
                    return;
                }
            };
            if page.gap {
                self.inner.event(
                    at,
                    GENOME_CHANGE_GAP_KIND,
                    "Some genome changes were evicted from the change ledger before they were read.",
                );
            }
            for change in &page.changes {
                let label = change_label(change, self.source);
                self.inner.event(at, GENOME_CHANGE_KIND, &label);
                if let Some(sequence) = change.get("sequence").and_then(Value::as_u64) {
                    since = since.max(sequence);
                }
            }
            if page.changes.is_empty() {
                since = since.max(page.latest_sequence);
            }
            if !page.has_more || page.changes.is_empty() {
                break;
            }
        }
        self.cursor = Some(since);
    }
}

impl RunDataSink for GenomeChangeSink<'_> {
    fn declare_table(&mut self, spec: &TableSpec) {
        if spec.name == self.table {
            self.column_index = spec.columns.iter().position(|c| c.name == self.column);
        }
        self.inner.declare_table(spec);
    }

    fn row(&mut self, table: &str, values: &[RunDataValue]) {
        self.inner.row(table, values);
        if table != self.table {
            return;
        }
        let episode = self
            .column_index
            .and_then(|i| values.get(i))
            .and_then(|v| match v {
                RunDataValue::Int(n) => u64::try_from(*n).ok(),
                _ => None,
            });
        self.collect(episode);
    }

    fn event(&mut self, at: EventTime, kind: &str, label: &str) {
        self.inner.event(at, kind, label);
    }
}

fn kind_word(kind: &str) -> &str {
    match kind {
        "cortical_area" => "area",
        "cortical_mapping" => "mapping",
        "brain_region" => "region",
        other => other,
    }
}

fn short_value(value: &Value) -> String {
    let text = match value {
        Value::Number(n) => match n.as_f64() {
            Some(f) if n.is_f64() => {
                let rounded = format!("{f:.4}");
                rounded
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            }
            _ => n.to_string(),
        },
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    truncate(&text, 24)
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Keys whose values differ, as `path old → new`, depth-limited.
fn diff(before: &Value, after: &Value, path: &str, depth: usize, out: &mut Vec<String>) {
    match (before, after) {
        (Value::Object(b), Value::Object(a)) if depth < 3 => {
            let mut keys: Vec<&String> = b.keys().chain(a.keys()).collect();
            keys.sort();
            keys.dedup();
            for key in keys {
                let child = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                diff(
                    b.get(key).unwrap_or(&Value::Null),
                    a.get(key).unwrap_or(&Value::Null),
                    &child,
                    depth + 1,
                    out,
                );
            }
        }
        _ if before == after => {}
        _ => {
            let name = if path.is_empty() { "value" } else { path };
            out.push(format!(
                "{name} {} → {}",
                short_value(before),
                short_value(after)
            ));
        }
    }
}

/// A short readable summary of one ledger entry: the target and only the keys that differ.
pub fn change_label(change: &Value, source: &mut dyn GenomeChangeSource) -> String {
    let targets = change.get("targets").and_then(Value::as_array);
    let first = targets.and_then(|t| t.first());
    let subject = match first {
        Some(target) => {
            let kind = target.get("kind").and_then(Value::as_str).unwrap_or("");
            // A mapping's ends are areas.
            let name_kind = if kind == "cortical_mapping" {
                "cortical_area"
            } else {
                kind
            };
            let name = |source: &mut dyn GenomeChangeSource, id: &str| {
                source
                    .display_name(name_kind, id)
                    .unwrap_or_else(|| id.to_string())
            };
            let id = target.get("id").and_then(Value::as_str).unwrap_or("");
            let mut text = name(source, id);
            if let Some(related) = target.get("related_id").and_then(Value::as_str) {
                text = format!("{text} → {}", name(source, related));
            }
            let more = targets.map_or(0, |t| t.len()).saturating_sub(1);
            if more > 0 {
                text.push_str(&format!(" +{more} more"));
            }
            format!("{} {text}", kind_word(kind))
        }
        None => "genome".to_string(),
    };
    let operation = change.get("operation");
    let op_kind = operation
        .and_then(|o| o.get("kind"))
        .and_then(Value::as_str)
        .unwrap_or("change");
    let mut parts = Vec::new();
    if let (Some(before), Some(after)) = (change.get("before"), change.get("after")) {
        diff(before, after, "", 0, &mut parts);
    }
    if parts.is_empty() {
        // No usable before/after: the requested keys, or the operation itself.
        let requested = operation
            .and_then(|o| o.get("changes").or_else(|| o.get("properties")))
            .and_then(Value::as_object);
        match requested {
            Some(map) if !map.is_empty() => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                parts.extend(
                    keys.into_iter()
                        .map(|k| format!("{k} = {}", short_value(&map[k]))),
                );
            }
            _ => parts.push(op_kind.replace('_', " ")),
        }
    }
    truncate(&format!("{subject}: {}", parts.join(", ")), LABEL_MAX_CHARS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_data::{ColumnSpec, RecordingRunDataSink};
    use serde_json::json;
    use std::collections::VecDeque;

    #[derive(Default)]
    struct FakeSource {
        pages: VecDeque<Result<ChangesPage, String>>,
        asked: Vec<u64>,
    }

    impl GenomeChangeSource for FakeSource {
        fn page(&mut self, since: u64, _limit: usize) -> Result<ChangesPage, String> {
            self.asked.push(since);
            self.pages
                .pop_front()
                .unwrap_or_else(|| Ok(ChangesPage::default()))
        }

        fn display_name(&mut self, kind: &str, id: &str) -> Option<String> {
            (kind == "cortical_area" && id == "id1").then(|| "alpha".to_string())
        }
    }

    fn area_change(sequence: u64, before: Value, after: Value) -> Value {
        json!({
            "sequence": sequence,
            "targets": [{"kind": "cortical_area", "id": "id1"}],
            "operation": {"kind": "update_cortical_area", "cortical_id": "id1", "changes": {}},
            "before": before,
            "after": after,
        })
    }

    fn page(changes: Vec<Value>, latest: u64, gap: bool, has_more: bool) -> ChangesPage {
        ChangesPage {
            changes,
            latest_sequence: latest,
            gap,
            has_more,
        }
    }

    fn attempts_spec() -> TableSpec {
        TableSpec {
            name: "attempts".to_string(),
            description: "d".to_string(),
            columns: vec![
                ColumnSpec::unitless("phase", "p"),
                ColumnSpec::unitless("attempt", "a"),
            ],
        }
    }

    fn attempt_row(index: i64) -> Vec<RunDataValue> {
        vec![RunDataValue::text("train"), RunDataValue::Int(index)]
    }

    #[test]
    fn one_event_per_change_tagged_with_the_attempt_and_only_differing_keys() {
        let mut inner = RecordingRunDataSink::default();
        let mut source = FakeSource::default();
        source.pages.push_back(Ok(page(vec![], 7, false, false)));
        source.pages.push_back(Ok(page(
            vec![
                area_change(
                    8,
                    json!({"leak": 0.1, "threshold": 1, "name": "alpha"}),
                    json!({"leak": 0.2, "threshold": 1, "name": "alpha"}),
                ),
                area_change(9, json!({"firing": 3}), json!({"firing": 4})),
            ],
            9,
            false,
            false,
        )));
        {
            let mut sink = GenomeChangeSink::new(&mut inner, &mut source, "attempts", "attempt");
            sink.declare_table(&attempts_spec());
            sink.row("attempts", &attempt_row(0));
            sink.row("attempts", &attempt_row(1));
            assert!(sink.failure().is_none());
        }
        assert_eq!(source.asked, vec![0, 7, 9]);
        let events = &inner.events;
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.kind == "genome_change"));
        assert!(events.iter().all(|e| e.at.episode == Some(0)));
        assert_eq!(events[0].label, "area alpha: leak 0.1 → 0.2");
        assert_eq!(events[1].label, "area alpha: firing 3 → 4");
        assert_eq!(inner.rows_of("attempts").len(), 2);
    }

    #[test]
    fn has_more_is_followed_and_a_gap_writes_one_gap_event() {
        let mut inner = RecordingRunDataSink::default();
        let mut source = FakeSource::default();
        source.pages.push_back(Ok(page(vec![], 0, false, false)));
        source.pages.push_back(Ok(page(
            vec![area_change(1, json!({"a": 1}), json!({"a": 2}))],
            2,
            true,
            true,
        )));
        source.pages.push_back(Ok(page(
            vec![area_change(2, json!({"a": 2}), json!({"a": 3}))],
            2,
            false,
            false,
        )));
        {
            let mut sink = GenomeChangeSink::new(&mut inner, &mut source, "attempts", "attempt");
            sink.declare_table(&attempts_spec());
            sink.row("attempts", &attempt_row(4));
        }
        assert_eq!(source.asked, vec![0, 0, 1]);
        assert_eq!(
            inner.event_kinds(),
            vec!["genome_change_gap", "genome_change", "genome_change"]
        );
        assert!(inner.events.iter().all(|e| e.at.episode == Some(4)));
    }

    #[test]
    fn endpoint_failure_is_recorded_once_and_the_rows_still_flow() {
        let mut inner = RecordingRunDataSink::default();
        let mut source = FakeSource::default();
        source.pages.push_back(Ok(page(vec![], 0, false, false)));
        source.pages.push_back(Err("HTTP 501".to_string()));
        {
            let mut sink = GenomeChangeSink::new(&mut inner, &mut source, "attempts", "attempt");
            sink.declare_table(&attempts_spec());
            sink.row("attempts", &attempt_row(0));
            sink.row("attempts", &attempt_row(1));
            assert_eq!(sink.failure(), Some("HTTP 501"));
        }
        assert_eq!(source.asked, vec![0, 0], "no call after the failure");
        assert!(inner.events.is_empty());
        assert_eq!(inner.rows_of("attempts").len(), 2);
    }

    #[test]
    fn unavailable_at_start_still_lets_the_run_through() {
        let mut inner = RecordingRunDataSink::default();
        let mut source = FakeSource::default();
        source.pages.push_back(Err("HTTP 501".to_string()));
        {
            let mut sink = GenomeChangeSink::new(&mut inner, &mut source, "attempts", "attempt");
            sink.declare_table(&attempts_spec());
            sink.row("attempts", &attempt_row(0));
            assert_eq!(sink.failure(), Some("HTTP 501"));
        }
        assert_eq!(source.asked, vec![0]);
        assert_eq!(inner.rows_of("attempts").len(), 1);
    }

    #[test]
    fn labels_cover_mappings_missing_snapshots_and_truncation() {
        let mut source = FakeSource::default();
        let mapping = json!({
            "targets": [{"kind": "cortical_mapping", "id": "id1", "related_id": "id2"}],
            "operation": {"kind": "update_cortical_mapping"},
            "before": [1], "after": [2],
        });
        assert_eq!(
            change_label(&mapping, &mut source),
            "mapping alpha → id2: value [1] → [2]"
        );
        let requested = json!({
            "targets": [{"kind": "cortical_area", "id": "id1"}],
            "operation": {"kind": "update_cortical_area", "changes": {"leak": 0.5}},
        });
        assert_eq!(
            change_label(&requested, &mut source),
            "area alpha: leak = 0.5"
        );
        let long = area_change(
            1,
            json!({"k": "a".repeat(200)}),
            json!({"k": "b".repeat(200)}),
        );
        assert!(change_label(&long, &mut source).chars().count() <= LABEL_MAX_CHARS);
    }
}
