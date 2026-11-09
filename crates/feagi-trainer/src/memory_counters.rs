//! Memory-area counters sampled at attempt boundaries.
//!
//! FEAGI publishes per-memory-area lifecycle totals (neurons created, patterns re-matched,
//! creations that failed, formations suppressed). A phase that is handed a
//! [`MemoryCounterSource`] samples it before its first attempt and after every attempt, and
//! writes one row per memory area and attempt to the declared `memory_areas` run-data table:
//! the running totals, the change over the attempt and the re-match rate. The source is
//! implemented over the FEAGI REST API by the app and by mocks in tests; this module never
//! touches the brain, so it cannot change what it measures.

use crate::run_data::{ColumnSpec, EventTime, RunDataSink, RunDataValue, TableSpec};

/// Name of the run-data table the phase writes.
pub const MEMORY_AREAS_TABLE: &str = "memory_areas";

/// Lifecycle totals of one memory area since FEAGI started counting.
///
/// A counter an older FEAGI does not report is `None`; its delta is then empty too.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAreaCounters {
    /// Area name (the cortical id when the area has no name).
    pub area: String,
    /// Memory neurons created.
    pub created_total: Option<u64>,
    /// Pattern presentations that matched an existing memory neuron.
    pub reactivated_total: Option<u64>,
    /// Creations that failed (no room).
    pub creation_failed_total: Option<u64>,
    /// Formations that were suppressed (for example by a learning freeze).
    pub formation_suppressed_total: Option<u64>,
}

/// The memory areas of the loaded genome and their counters right now.
pub trait MemoryCounterSource {
    /// One entry per memory area, in a stable order. An empty list means the genome has no
    /// memory area.
    fn sample(&mut self) -> Result<Vec<MemoryAreaCounters>, crate::error::TrainerError>;
}

/// Share of pattern presentations that matched an existing neuron over a window:
/// `reactivated / (reactivated + created + creation_failed + formation_suppressed)`. Empty
/// when no presentation was counted or the reactivation or creation counter is missing. The
/// failure and suppression counters count as 0 when an older FEAGI lacks them.
pub fn rematch_rate(
    reactivated: Option<u64>,
    created: Option<u64>,
    creation_failed: Option<u64>,
    formation_suppressed: Option<u64>,
) -> Option<f64> {
    let total = reactivated?
        .saturating_add(created?)
        .saturating_add(creation_failed.unwrap_or(0))
        .saturating_add(formation_suppressed.unwrap_or(0));
    (total > 0).then(|| reactivated.unwrap_or(0) as f64 / total as f64)
}

fn delta(before: Option<u64>, after: Option<u64>) -> Option<u64> {
    Some(after?.saturating_sub(before?))
}

/// The `memory_areas` table: one row per memory area per attempt.
pub fn memory_areas_table_spec() -> TableSpec {
    TableSpec {
        name: MEMORY_AREAS_TABLE.to_string(),
        description: "One row per memory area per finished attempt: FEAGI's lifecycle totals at the \
                      end of the attempt, the change over the attempt and the re-match rate."
            .to_string(),
        columns: vec![
            ColumnSpec::unitless("phase", "Phase the attempt belongs to."),
            ColumnSpec::unitless("attempt", "Attempt index within the phase."),
            ColumnSpec::unitless("area", "Memory area name."),
            ColumnSpec::unitless(
                "created_total",
                "Memory neurons created since FEAGI started counting, at the end of the attempt.",
            ),
            ColumnSpec::unitless(
                "reactivated_total",
                "Pattern presentations that matched an existing memory neuron, since FEAGI started counting.",
            ),
            ColumnSpec::unitless(
                "creation_failed_total",
                "Memory neuron creations that failed, since FEAGI started counting.",
            ),
            ColumnSpec::unitless(
                "formation_suppressed_total",
                "Memory neuron formations that were suppressed, since FEAGI started counting.",
            ),
            ColumnSpec::unitless(
                "created_delta",
                "Memory neurons created during the attempt (growth).",
            ),
            ColumnSpec::unitless(
                "reactivated_delta",
                "Pattern presentations that matched an existing memory neuron during the attempt.",
            ),
            ColumnSpec::unitless(
                "creation_failed_delta",
                "Memory neuron creations that failed during the attempt.",
            ),
            ColumnSpec::unitless(
                "formation_suppressed_delta",
                "Memory neuron formations suppressed during the attempt.",
            ),
            ColumnSpec::unitless(
                "rematch_rate",
                "reactivated / (reactivated + created + creation_failed + formation_suppressed) over the attempt. Empty when no pattern was presented or a counter is missing.",
            ),
        ],
    }
}

/// Samples a source at attempt boundaries and writes the table. A source that fails once is
/// dropped for the rest of the phase and the failure is logged as an event, so a missing
/// reading never fails the run.
pub struct MemoryAreasRecorder<'a> {
    source: &'a mut dyn MemoryCounterSource,
    previous: Option<Vec<MemoryAreaCounters>>,
    failed: bool,
}

impl<'a> MemoryAreasRecorder<'a> {
    /// Declare the table and take the first sample, the baseline of attempt 1.
    pub fn start(source: &'a mut dyn MemoryCounterSource, sink: &mut dyn RunDataSink) -> Self {
        sink.declare_table(&memory_areas_table_spec());
        let mut recorder = Self {
            source,
            previous: None,
            failed: false,
        };
        recorder.previous = recorder.read(sink);
        recorder
    }

    fn read(&mut self, sink: &mut dyn RunDataSink) -> Option<Vec<MemoryAreaCounters>> {
        if self.failed {
            return None;
        }
        match self.source.sample() {
            Ok(areas) => Some(areas),
            Err(error) => {
                self.failed = true;
                sink.event(
                    EventTime::default(),
                    "memory_counters_unavailable",
                    &error.to_string(),
                );
                None
            }
        }
    }

    /// Sample again and write one row per memory area: totals now, change since the previous
    /// sample, re-match rate. `attempt` is the 1-based index of the attempt that just ended.
    pub fn after_attempt(&mut self, sink: &mut dyn RunDataSink, phase: &str, attempt: usize) {
        let Some(now) = self.read(sink) else {
            return;
        };
        let before = self.previous.take().unwrap_or_default();
        for area in &now {
            let was = before.iter().find(|b| b.area == area.area);
            let d_created = delta(was.and_then(|w| w.created_total), area.created_total);
            let d_reactivated = delta(
                was.and_then(|w| w.reactivated_total),
                area.reactivated_total,
            );
            let d_failed = delta(
                was.and_then(|w| w.creation_failed_total),
                area.creation_failed_total,
            );
            let d_suppressed = delta(
                was.and_then(|w| w.formation_suppressed_total),
                area.formation_suppressed_total,
            );
            let count = |v: Option<u64>| v.map_or(RunDataValue::Null, RunDataValue::count);
            sink.row(
                MEMORY_AREAS_TABLE,
                &[
                    RunDataValue::text(phase),
                    RunDataValue::count(attempt as u64),
                    RunDataValue::text(&area.area),
                    count(area.created_total),
                    count(area.reactivated_total),
                    count(area.creation_failed_total),
                    count(area.formation_suppressed_total),
                    count(d_created),
                    count(d_reactivated),
                    count(d_failed),
                    count(d_suppressed),
                    RunDataValue::opt_float(rematch_rate(
                        d_reactivated,
                        d_created,
                        d_failed,
                        d_suppressed,
                    )),
                ],
            );
        }
        self.previous = Some(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::TrainerError;
    use crate::run_data::RecordingRunDataSink;

    struct Script(Vec<Result<Vec<MemoryAreaCounters>, String>>);

    impl MemoryCounterSource for Script {
        fn sample(&mut self) -> Result<Vec<MemoryAreaCounters>, TrainerError> {
            self.0.remove(0).map_err(TrainerError::Runtime)
        }
    }

    fn area(created: u64, reactivated: u64) -> Vec<MemoryAreaCounters> {
        vec![MemoryAreaCounters {
            area: "mem".to_string(),
            created_total: Some(created),
            reactivated_total: Some(reactivated),
            creation_failed_total: Some(0),
            formation_suppressed_total: None,
        }]
    }

    #[test]
    fn rematch_rate_counts_reactivations_over_all_presentations() {
        assert_eq!(rematch_rate(Some(3), Some(1), Some(0), None), Some(0.75));
        assert_eq!(rematch_rate(Some(0), Some(0), Some(0), Some(0)), None);
        assert_eq!(rematch_rate(None, Some(1), None, None), None);
        assert_eq!(rematch_rate(Some(2), Some(0), Some(1), Some(1)), Some(0.5));
    }

    #[test]
    fn each_attempt_writes_totals_deltas_and_rate_per_area() {
        let mut source = Script(vec![Ok(area(5, 10)), Ok(area(6, 13)), Ok(area(6, 13))]);
        let mut sink = RecordingRunDataSink::default();
        let mut recorder = MemoryAreasRecorder::start(&mut source, &mut sink);
        recorder.after_attempt(&mut sink, "observe", 1);
        recorder.after_attempt(&mut sink, "observe", 2);
        assert!(sink.errors.is_empty(), "{:?}", sink.errors);
        let rows = sink.rows_of(MEMORY_AREAS_TABLE);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], RunDataValue::text("observe"));
        assert_eq!(rows[0][1], RunDataValue::count(1));
        assert_eq!(rows[0][2], RunDataValue::text("mem"));
        assert_eq!(rows[0][3], RunDataValue::count(6));
        assert_eq!(rows[0][7], RunDataValue::count(1));
        assert_eq!(rows[0][8], RunDataValue::count(3));
        assert_eq!(rows[0][10], RunDataValue::Null);
        assert_eq!(rows[0][11], RunDataValue::Float(0.75));
        // Nothing changed in attempt 2: no presentation, so no rate.
        assert_eq!(rows[1][7], RunDataValue::count(0));
        assert_eq!(rows[1][11], RunDataValue::Null);
    }

    #[test]
    fn a_failed_sample_is_logged_once_and_never_fails_the_phase() {
        let mut source = Script(vec![Ok(area(1, 1)), Err("health_check down".to_string())]);
        let mut sink = RecordingRunDataSink::default();
        let mut recorder = MemoryAreasRecorder::start(&mut source, &mut sink);
        recorder.after_attempt(&mut sink, "observe", 1);
        recorder.after_attempt(&mut sink, "observe", 2);
        assert!(sink.rows_of(MEMORY_AREAS_TABLE).is_empty());
        let events: Vec<_> = sink
            .events
            .iter()
            .filter(|e| e.kind == "memory_counters_unavailable")
            .collect();
        assert_eq!(events.len(), 1);
        assert!(events[0].label.contains("health_check down"));
    }

    #[test]
    fn a_genome_without_memory_areas_writes_no_rows() {
        let mut source = Script(vec![Ok(Vec::new()), Ok(Vec::new())]);
        let mut sink = RecordingRunDataSink::default();
        let mut recorder = MemoryAreasRecorder::start(&mut source, &mut sink);
        recorder.after_attempt(&mut sink, "observe", 1);
        assert!(sink.rows_of(MEMORY_AREAS_TABLE).is_empty());
        assert_eq!(sink.tables.len(), 1);
    }
}
