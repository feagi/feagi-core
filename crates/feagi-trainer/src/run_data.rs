//! Run data sink: the dataset every Trainer run can produce.
//!
//! A run writes **declared tables** of per-step or per-sample values, plus an **event log**,
//! through a [`RunDataSink`]. A host turns that into files, a manifest and a data dictionary.
//!
//! # Contract
//!
//! - A table is declared (name, description, ordered columns with optional units) before its
//!   first row. Declaring the same name again with identical columns is a no-op.
//! - A row carries one [`RunDataValue`] per declared column, in declaration order.
//! - An event carries a time stamp ([`EventTime`], every part optional), a `kind` and a
//!   `label`. Event kinds and labels are data chosen by whoever produces them. The sink adds
//!   no meaning to them and does not validate or interpret them.
//! - The sink never fails a run: a sink that cannot store a value records the problem itself.

/// One column of a table.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnSpec {
    /// Column name, unique within the table.
    pub name: String,
    /// Unit of the values, when they have one.
    pub unit: Option<String>,
    /// What the column holds.
    pub description: String,
}

impl ColumnSpec {
    /// A column with a unit.
    pub fn with_unit(name: &str, unit: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            unit: Some(unit.to_string()),
            description: description.to_string(),
        }
    }

    /// A column without a unit.
    pub fn unitless(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            unit: None,
            description: description.to_string(),
        }
    }
}

/// A table: a name, what it holds and its ordered columns.
#[derive(Debug, Clone, PartialEq)]
pub struct TableSpec {
    /// Table name, unique within a run.
    pub name: String,
    /// What one row of the table is.
    pub description: String,
    /// Columns in row order.
    pub columns: Vec<ColumnSpec>,
}

/// One cell.
#[derive(Debug, Clone, PartialEq)]
pub enum RunDataValue {
    /// Real number.
    Float(f64),
    /// Integer.
    Int(i64),
    /// Flag.
    Bool(bool),
    /// Text.
    Text(String),
    /// No value.
    Null,
}

impl RunDataValue {
    /// A float, or [`RunDataValue::Null`] when absent.
    pub fn opt_float(value: Option<f64>) -> Self {
        value.map_or(Self::Null, Self::Float)
    }

    /// An integer from an unsigned count.
    pub fn count(value: u64) -> Self {
        Self::Int(i64::try_from(value).unwrap_or(i64::MAX))
    }

    /// A text cell.
    pub fn text(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

/// The serialized name of an enum value (its serde string form), for text cells.
pub(crate) fn name_of<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// When an event happened. Every part is optional; a producer sets what it knows.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EventTime {
    /// Simulated seconds.
    pub sim_time_s: Option<f64>,
    /// Step or sample counter.
    pub step: Option<u64>,
    /// Episode, attempt or repeat counter.
    pub episode: Option<u64>,
}

/// Receives the tables and events of one run.
pub trait RunDataSink {
    /// Declare a table before its first row. Declaring the same name with the same columns
    /// again is a no-op.
    fn declare_table(&mut self, spec: &TableSpec);

    /// Append a row to a declared table. `values` follow the declared column order.
    fn row(&mut self, table: &str, values: &[RunDataValue]);

    /// Append an event to the log.
    fn event(&mut self, at: EventTime, kind: &str, label: &str);
}

/// Drops everything.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopRunDataSink;

impl RunDataSink for NoopRunDataSink {
    fn declare_table(&mut self, _spec: &TableSpec) {}
    fn row(&mut self, _table: &str, _values: &[RunDataValue]) {}
    fn event(&mut self, _at: EventTime, _kind: &str, _label: &str) {}
}

/// One recorded event.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedEvent {
    /// When it happened.
    pub at: EventTime,
    /// Kind, as given.
    pub kind: String,
    /// Label, as given.
    pub label: String,
}

/// In-memory sink for tests and small hosts. A row whose length differs from its declaration,
/// a row for an undeclared table and a conflicting re-declaration are recorded in
/// [`errors`](Self::errors) and never panic.
#[derive(Debug, Default, Clone)]
pub struct RecordingRunDataSink {
    /// Declared tables, in declaration order.
    pub tables: Vec<TableSpec>,
    /// Rows by table name, in arrival order.
    pub rows: std::collections::BTreeMap<String, Vec<Vec<RunDataValue>>>,
    /// Events in arrival order.
    pub events: Vec<RecordedEvent>,
    /// Contract violations seen.
    pub errors: Vec<String>,
}

impl RecordingRunDataSink {
    /// Rows of one table (empty when none).
    pub fn rows_of(&self, table: &str) -> &[Vec<RunDataValue>] {
        self.rows.get(table).map(Vec::as_slice).unwrap_or(&[])
    }

    /// The declaration of one table.
    pub fn table(&self, name: &str) -> Option<&TableSpec> {
        self.tables.iter().find(|spec| spec.name == name)
    }

    /// Index of a column in a declared table.
    pub fn column_index(&self, table: &str, column: &str) -> Option<usize> {
        self.table(table)?
            .columns
            .iter()
            .position(|spec| spec.name == column)
    }

    /// Kinds of the events, in arrival order.
    pub fn event_kinds(&self) -> Vec<&str> {
        self.events.iter().map(|e| e.kind.as_str()).collect()
    }
}

impl RunDataSink for RecordingRunDataSink {
    fn declare_table(&mut self, spec: &TableSpec) {
        match self.tables.iter().find(|known| known.name == spec.name) {
            Some(known) if known.columns == spec.columns => {}
            Some(_) => self.errors.push(format!(
                "table {:?} re-declared with different columns",
                spec.name
            )),
            None => self.tables.push(spec.clone()),
        }
    }

    fn row(&mut self, table: &str, values: &[RunDataValue]) {
        let Some(spec) = self.tables.iter().find(|spec| spec.name == table) else {
            self.errors
                .push(format!("row for undeclared table {table:?}"));
            return;
        };
        if spec.columns.len() != values.len() {
            self.errors.push(format!(
                "table {table:?}: row has {} values for {} columns",
                values.len(),
                spec.columns.len()
            ));
            return;
        }
        self.rows
            .entry(table.to_string())
            .or_default()
            .push(values.to_vec());
    }

    fn event(&mut self, at: EventTime, kind: &str, label: &str) {
        self.events.push(RecordedEvent {
            at,
            kind: kind.to_string(),
            label: label.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(columns: &[&str]) -> TableSpec {
        TableSpec {
            name: "t".to_string(),
            description: "d".to_string(),
            columns: columns
                .iter()
                .map(|name| ColumnSpec::unitless(name, "c"))
                .collect(),
        }
    }

    #[test]
    fn declared_table_takes_rows_of_matching_length() {
        let mut sink = RecordingRunDataSink::default();
        sink.declare_table(&spec(&["a", "b"]));
        sink.row("t", &[RunDataValue::Int(1), RunDataValue::Null]);
        assert_eq!(sink.rows_of("t").len(), 1);
        assert_eq!(sink.column_index("t", "b"), Some(1));
        assert!(sink.errors.is_empty());
    }

    #[test]
    fn wrong_length_and_undeclared_rows_are_errors_not_panics() {
        let mut sink = RecordingRunDataSink::default();
        sink.declare_table(&spec(&["a", "b"]));
        sink.row("t", &[RunDataValue::Int(1)]);
        sink.row("missing", &[]);
        assert!(sink.rows_of("t").is_empty());
        assert_eq!(sink.errors.len(), 2);
    }

    #[test]
    fn redeclare_same_columns_is_a_noop_and_different_is_an_error() {
        let mut sink = RecordingRunDataSink::default();
        sink.declare_table(&spec(&["a"]));
        sink.declare_table(&spec(&["a"]));
        assert_eq!(sink.tables.len(), 1);
        assert!(sink.errors.is_empty());
        sink.declare_table(&spec(&["a", "b"]));
        assert_eq!(sink.tables.len(), 1);
        assert_eq!(sink.errors.len(), 1);
    }

    #[test]
    fn events_keep_arrival_order_and_text_as_given() {
        let mut sink = RecordingRunDataSink::default();
        let at = EventTime {
            sim_time_s: Some(1.5),
            ..EventTime::default()
        };
        sink.event(at, "k1", "x");
        sink.event(EventTime::default(), "k2", "");
        assert_eq!(sink.event_kinds(), vec!["k1", "k2"]);
        assert_eq!(sink.events[0].at.sim_time_s, Some(1.5));
        NoopRunDataSink.event(at, "k", "l");
    }
}
