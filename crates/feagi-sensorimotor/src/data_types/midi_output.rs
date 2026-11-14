//! MIDI output frames and their registration properties.
//!
//! A MIDI Output unit is one instrument, made of two cortical areas. Several
//! instruments are several units of this type (unit index 0, 1, 2, ...); the
//! number of units is not capped here.
//!
//! - **Notes**: X is one column per note (low to high), Y and Z are 1. A voxel
//!   that fires this frame is a sounding note; its strength is its potential on
//!   FEAGI's 0..=255 scale divided by 255.
//! - **Expression**: X is one column per [`MidiControl`], Y is 1, Z is the level
//!   of that control (unsigned percentage coding along Z).

use feagi_structures::FeagiDataError;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Largest number of note columns (template X maximum of the Notes area).
pub const MIDI_OUTPUT_MAX_NOTE_COLUMNS: u32 = 128;
/// Default Z depth of the Expression area, the same as other percentage outputs.
pub const MIDI_OUTPUT_DEFAULT_EXPRESSION_DEPTH: u32 = 10;
/// Largest Z depth of the Expression area (template Z maximum).
pub const MIDI_OUTPUT_MAX_EXPRESSION_DEPTH: u32 = 1024;
/// Full-scale membrane potential for a Notes strength of 1.0. This is FEAGI's
/// native membrane level scale (levels `0..=255`, `MEMBRANE_MAX` in
/// `feagi-npu/neural/src/types/numeric.rs`); potentials above it saturate at
/// strength 1.0. Defined here because this crate does not depend on the neural
/// crate.
pub const MIDI_OUTPUT_NOTE_FULL_SCALE_POTENTIAL: f32 = 255.0;

/// One column of the Expression area. The discriminant is the X column, in
/// this fixed order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MidiControl {
    /// Column 0. Antagonist of [`MidiControl::PitchBendDown`]; together they
    /// carry the signed pitch bend.
    PitchBendUp = 0,
    /// Column 1. Antagonist of [`MidiControl::PitchBendUp`].
    PitchBendDown = 1,
    /// Column 2. Modulation depth.
    Modulation = 2,
    /// Column 3. Sustain level.
    Sustain = 3,
    /// Column 4. Volume level.
    Volume = 4,
}

impl MidiControl {
    /// Number of controls, which is the X width of the Expression area.
    pub const COUNT: usize = 5;

    /// Every control in column order.
    pub const ALL: [MidiControl; MidiControl::COUNT] = [
        MidiControl::PitchBendUp,
        MidiControl::PitchBendDown,
        MidiControl::Modulation,
        MidiControl::Sustain,
        MidiControl::Volume,
    ];

    /// The Expression X column of this control.
    pub const fn column(self) -> usize {
        self as usize
    }

    /// The control at an Expression X column, if any.
    pub const fn from_column(column: usize) -> Option<MidiControl> {
        match column {
            0 => Some(MidiControl::PitchBendUp),
            1 => Some(MidiControl::PitchBendDown),
            2 => Some(MidiControl::Modulation),
            3 => Some(MidiControl::Sustain),
            4 => Some(MidiControl::Volume),
            _ => None,
        }
    }
}

/// Registration properties of a MIDI Output unit (one instrument).
///
/// Dims contract: Notes is `note_columns x 1 x 1` per channel and Expression is
/// `5 x 1 x expression_depth` per channel (`5` is [`MidiControl::COUNT`]).
/// Carried in the agent's device registration so FEAGI sizes both areas from
/// it, the same way other sized outputs are sized. Several instruments are
/// several units, each with its own properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MidiOutputProperties {
    /// Note columns per channel (Notes area X width), 1 to 128.
    pub note_columns: u32,
    /// Z depth of the Expression area; the resolution of each control level.
    pub expression_depth: u32,
}

impl MidiOutputProperties {
    pub fn new(note_columns: u32, expression_depth: u32) -> Result<Self, FeagiDataError> {
        let properties = MidiOutputProperties {
            note_columns,
            expression_depth,
        };
        properties.validate()?;
        Ok(properties)
    }

    pub fn validate(&self) -> Result<(), FeagiDataError> {
        let bad = |message: String| Err(FeagiDataError::BadParameters(message));
        if self.note_columns == 0 || self.note_columns > MIDI_OUTPUT_MAX_NOTE_COLUMNS {
            return bad(format!(
                "MIDI note columns {} must be between 1 and {}",
                self.note_columns, MIDI_OUTPUT_MAX_NOTE_COLUMNS
            ));
        }
        if self.expression_depth == 0 || self.expression_depth > MIDI_OUTPUT_MAX_EXPRESSION_DEPTH {
            return bad(format!(
                "MIDI expression depth {} must be between 1 and {}",
                self.expression_depth, MIDI_OUTPUT_MAX_EXPRESSION_DEPTH
            ));
        }
        Ok(())
    }
}

impl Display for MidiOutputProperties {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MidiOutput({} notes, expression depth {})",
            self.note_columns, self.expression_depth
        )
    }
}

/// Decoded Notes area of one instrument: one strength per note column.
///
/// Strength 0.0 means "not firing this frame", that is, not sounding. A firing
/// voxel's strength is its membrane potential at fire on FEAGI's native
/// `0..=255` level scale, divided by [`MIDI_OUTPUT_NOTE_FULL_SCALE_POTENTIAL`]
/// and clamped to `0..=1` (potentials above 255 saturate). An ordinary
/// connection delivers its strength as the target's potential, and a default
/// forced stimulation (potential 100) reads as 100/255. Potential at an output
/// is never below the neuron's firing threshold. Every decode replaces the
/// whole state.
#[derive(Clone, Debug, PartialEq)]
pub struct MidiNotesFrame {
    note_columns: usize,
    strengths: Vec<f32>,
}

impl MidiNotesFrame {
    /// A silent frame (all zeros).
    pub fn new(properties: &MidiOutputProperties) -> Result<Self, FeagiDataError> {
        properties.validate()?;
        let note_columns = properties.note_columns as usize;
        Ok(MidiNotesFrame {
            note_columns,
            strengths: vec![0.0; note_columns],
        })
    }

    pub fn note_columns(&self) -> usize {
        self.note_columns
    }

    /// Strength at a note column; 0.0 when silent or out of range.
    pub fn strength(&self, column: usize) -> f32 {
        self.strengths.get(column).copied().unwrap_or(0.0)
    }

    /// All strengths, low note to high.
    pub fn strengths(&self) -> &[f32] {
        &self.strengths
    }

    /// Set every strength to 0.0.
    pub fn silence(&mut self) {
        self.strengths.fill(0.0);
    }

    /// Keep the larger of the stored strength and the given potential (0..=255
    /// scale) converted to a strength in 0..=1. A potential at or below 0, or
    /// not finite, is not sounding. Out of range columns are ignored.
    pub(crate) fn raise_strength(&mut self, column: usize, potential: f32) {
        if !potential.is_finite() {
            return;
        }
        let strength = (potential / MIDI_OUTPUT_NOTE_FULL_SCALE_POTENTIAL).clamp(0.0, 1.0);
        if let Some(cell) = self.strengths.get_mut(column) {
            if strength > *cell {
                *cell = strength;
            }
        }
    }

    /// Number of sounding notes (strength above 0).
    pub fn sounding_count(&self) -> usize {
        self.strengths.iter().filter(|s| **s > 0.0).count()
    }
}

impl Display for MidiNotesFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MidiNotesFrame({} sounding of {})",
            self.sounding_count(),
            self.note_columns
        )
    }
}

/// Decoded Expression area of one instrument: one optional level per control.
///
/// `Some(level)` (0..1) means that control column fired this frame; `None`
/// means it was silent, that is, not driven. The decoder does not decide what
/// silence means for a control (hold, release or default); the connector does.
/// Every decode replaces the whole state.
#[derive(Clone, Debug, PartialEq)]
pub struct MidiExpressionFrame {
    /// Indexed by [`MidiControl::column`].
    levels: [Option<f32>; MidiControl::COUNT],
}

impl MidiExpressionFrame {
    /// A frame with every control silent (`None`).
    pub fn new(properties: &MidiOutputProperties) -> Result<Self, FeagiDataError> {
        properties.validate()?;
        Ok(MidiExpressionFrame {
            levels: [None; MidiControl::COUNT],
        })
    }

    /// Level of a control; `None` when silent.
    pub fn level(&self, control: MidiControl) -> Option<f32> {
        self.levels[control.column()]
    }

    /// The levels in [`MidiControl`] column order.
    pub fn levels(&self) -> &[Option<f32>] {
        &self.levels
    }

    /// Set every control to `None`.
    pub fn silence(&mut self) {
        self.levels = [None; MidiControl::COUNT];
    }

    pub(crate) fn set_level(&mut self, control: MidiControl, level: f32) {
        self.levels[control.column()] = Some(level);
    }

    /// Number of controls that fired this frame.
    pub fn driven_count(&self) -> usize {
        self.levels.iter().filter(|l| l.is_some()).count()
    }
}

impl Display for MidiExpressionFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MidiExpressionFrame({} driven of {})",
            self.driven_count(),
            MidiControl::COUNT
        )
    }
}

/// One decoded MIDI Output frame: both areas of the unit for one channel.
///
/// Silence means "not sounding" (Notes) and "not driven" (Expression).
#[derive(Clone, Debug, PartialEq)]
pub struct MidiOutputFrame {
    properties: MidiOutputProperties,
    pub notes: MidiNotesFrame,
    pub expression: MidiExpressionFrame,
}

impl MidiOutputFrame {
    /// A fully silent frame.
    pub fn new(properties: &MidiOutputProperties) -> Result<Self, FeagiDataError> {
        Ok(MidiOutputFrame {
            properties: *properties,
            notes: MidiNotesFrame::new(properties)?,
            expression: MidiExpressionFrame::new(properties)?,
        })
    }

    pub fn get_properties(&self) -> &MidiOutputProperties {
        &self.properties
    }

    /// Silence both areas.
    pub fn silence(&mut self) {
        self.notes.silence();
        self.expression.silence();
    }
}

impl Display for MidiOutputFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiOutputFrame({}, {})", self.notes, self.expression)
    }
}
