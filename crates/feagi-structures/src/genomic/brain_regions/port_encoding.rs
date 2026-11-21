// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Brain-region `port_encoding` property: what each designated port carries.

Sibling of `designated_inputs` / `designated_outputs`. Stored on the region as
`{ "<designated area id, base64>": { "kind", "signed", "frame", "positioning", "source" } }`.

* `kind`, `frame` and `positioning` are the vocabulary of the block datasheet
  (`KINDS`, `FRAMES`, `POSITIONINGS` in Composer's `block_datasheet.py`); `signed` is a boolean.
  Any of the four may be `null` (unknown).
* `source` is `"wiring"` when feagi derived the entry from the IPU/OPU areas wired to the port, and
  `"author"` when a person set it. Wiring derivation never overwrites an `"author"` entry.

Flag decoding is [`IOCorticalAreaConfigurationFlag::port_encoding_fields`]; the shared table
`tests/io-flag-fixture.json` keeps it identical to Composer and feagi-mcp.
*/

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::genomic::brain_regions::BrainRegion;
use crate::genomic::cortical_area::io_cortical_area_configuration_flag::PortEncodingFields;
use crate::genomic::cortical_area::CorticalID;
use crate::FeagiDataError;

/// Region property key holding the port encoding map.
pub const PORT_ENCODING_KEY: &str = "port_encoding";

/// Allowed `kind` values (same order as the IO configuration flag variants).
pub const PORT_KINDS: [&str; 12] = [
    "Boolean",
    "Percentage",
    "Percentage2D",
    "Percentage3D",
    "Percentage4D",
    "SignedPercentage",
    "SignedPercentage2D",
    "SignedPercentage3D",
    "SignedPercentage4D",
    "CartesianPlane",
    "Misc",
    "PoseEstimation",
];
/// Allowed `frame` values.
pub const PORT_FRAMES: [&str; 2] = ["Absolute", "Incremental"];
/// Allowed `positioning` values.
pub const PORT_POSITIONINGS: [&str; 2] = ["Linear", "Fractional"];

const KIND_FIELD: &str = "kind";
const SIGNED_FIELD: &str = "signed";
const FRAME_FIELD: &str = "frame";
const POSITIONING_FIELD: &str = "positioning";
const SOURCE_FIELD: &str = "source";
const SOURCE_WIRING: &str = "wiring";
const SOURCE_AUTHOR: &str = "author";

/// Where a port encoding entry came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortEncodingSource {
    /// Derived from the IPU/OPU areas wired to the port (or the port's own IO flags).
    Wiring,
    /// Set by a person through the region API.
    Author,
}

impl PortEncodingSource {
    /// Wire text of the source.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wiring => SOURCE_WIRING,
            Self::Author => SOURCE_AUTHOR,
        }
    }

    fn parse(text: &str) -> Result<Self, FeagiDataError> {
        match text {
            SOURCE_WIRING => Ok(Self::Wiring),
            SOURCE_AUTHOR => Ok(Self::Author),
            other => Err(bad(format!(
                "port_encoding source '{other}' is not one of: {SOURCE_WIRING}, {SOURCE_AUTHOR}"
            ))),
        }
    }
}

/// One designated port's encoding. A `None` field is unknown (`null` on the wire).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortEncoding {
    pub kind: Option<String>,
    pub signed: Option<bool>,
    pub frame: Option<String>,
    pub positioning: Option<String>,
    pub source: PortEncodingSource,
}

fn bad(message: String) -> FeagiDataError {
    FeagiDataError::BadParameters(message)
}

/// Read a nullable vocabulary field: absent or `null` is `None`, otherwise one of `allowed`.
fn vocabulary_field(
    object: &Map<String, Value>,
    field: &str,
    allowed: &[&str],
) -> Result<Option<String>, FeagiDataError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if allowed.contains(&text.as_str()) => Ok(Some(text.clone())),
        Some(other) => Err(bad(format!(
            "port_encoding {field} {other} is not one of: {}",
            allowed.join(", ")
        ))),
    }
}

/// Reject object keys that are not one of `known`.
fn reject_unknown_keys(object: &Map<String, Value>, known: &[&str]) -> Result<(), FeagiDataError> {
    for key in object.keys() {
        if !known.contains(&key.as_str()) {
            return Err(bad(format!(
                "port_encoding entry has unknown field '{key}' (allowed: {})",
                known.join(", ")
            )));
        }
    }
    Ok(())
}

impl PortEncoding {
    fn from_object(
        value: &Value,
        source: PortEncodingSource,
        known: &[&str],
    ) -> Result<Self, FeagiDataError> {
        let object = value
            .as_object()
            .ok_or_else(|| bad("a port_encoding entry must be a JSON object".to_string()))?;
        reject_unknown_keys(object, known)?;
        let signed = match object.get(SIGNED_FIELD) {
            None | Some(Value::Null) => None,
            Some(Value::Bool(flag)) => Some(*flag),
            Some(other) => {
                return Err(bad(format!(
                    "port_encoding signed {other} must be true, false or null"
                )))
            }
        };
        Ok(Self {
            kind: vocabulary_field(object, KIND_FIELD, &PORT_KINDS)?,
            signed,
            frame: vocabulary_field(object, FRAME_FIELD, &PORT_FRAMES)?,
            positioning: vocabulary_field(object, POSITIONING_FIELD, &PORT_POSITIONINGS)?,
            source,
        })
    }

    /// Parse a stored entry; `source` is required and must be `"wiring"` or `"author"`.
    pub fn from_stored_value(value: &Value) -> Result<Self, FeagiDataError> {
        let source = value
            .get(SOURCE_FIELD)
            .and_then(Value::as_str)
            .ok_or_else(|| bad("a stored port_encoding entry needs a string source".to_string()))
            .and_then(PortEncodingSource::parse)?;
        Self::from_object(
            value,
            source,
            &[
                KIND_FIELD,
                SIGNED_FIELD,
                FRAME_FIELD,
                POSITIONING_FIELD,
                SOURCE_FIELD,
            ],
        )
    }

    /// Parse an entry an author sends through the API. The server assigns `source = "author"`,
    /// so a `source` key in the input is rejected.
    pub fn from_author_value(value: &Value) -> Result<Self, FeagiDataError> {
        Self::from_object(
            value,
            PortEncodingSource::Author,
            &[KIND_FIELD, SIGNED_FIELD, FRAME_FIELD, POSITIONING_FIELD],
        )
    }

    /// Wire form: all five keys, unknown fields as `null`.
    pub fn to_value(&self) -> Value {
        json!({
            KIND_FIELD: self.kind,
            SIGNED_FIELD: self.signed,
            FRAME_FIELD: self.frame,
            POSITIONING_FIELD: self.positioning,
            SOURCE_FIELD: self.source.as_str(),
        })
    }

    /// Wiring entry holding exactly what one IO flag defines.
    pub fn from_flag_fields(fields: &PortEncodingFields) -> Self {
        Self {
            kind: Some(fields.kind.to_string()),
            signed: fields.signed,
            frame: fields.frame.map(|frame| frame.label().to_string()),
            positioning: fields.positioning.map(|p| p.label().to_string()),
            source: PortEncodingSource::Wiring,
        }
    }

    /// Wiring entry for several IO peers: a field the peers disagree on is `None`, an agreed
    /// field is kept. `None` when there are no peers.
    pub fn from_peer_fields(peers: &[PortEncodingFields]) -> Option<Self> {
        let (first, rest) = peers.split_first()?;
        let mut merged = Self::from_flag_fields(first);
        for peer in rest {
            let other = Self::from_flag_fields(peer);
            if merged.kind != other.kind {
                merged.kind = None;
            }
            if merged.signed != other.signed {
                merged.signed = None;
            }
            if merged.frame != other.frame {
                merged.frame = None;
            }
            if merged.positioning != other.positioning {
                merged.positioning = None;
            }
        }
        Some(merged)
    }
}

/// Which side of a region an IO area id belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoPortSide {
    /// IPU id (leading byte `i`).
    Input,
    /// OPU id (leading byte `o`).
    Output,
}

/// Result of decoding a cortical id as an IO area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoAreaDecode {
    /// Not an IPU or OPU id.
    NotIo,
    /// An IPU/OPU id whose flag the flag decoder refuses.
    Refused(IoPortSide),
    /// An IPU/OPU id and the fields its flag defines.
    Fields(IoPortSide, PortEncodingFields),
}

/// Decode an area id the same way Composer's `io_flag_decode.area_fields` and Studio's
/// `region_port_hints` do.
pub fn decode_io_area(id: &CorticalID) -> IoAreaDecode {
    let side = match id.as_bytes()[0] {
        b'i' => IoPortSide::Input,
        b'o' => IoPortSide::Output,
        _ => return IoAreaDecode::NotIo,
    };
    match id.extract_io_data_flag() {
        Ok(flag) => IoAreaDecode::Fields(side, flag.port_encoding_fields()),
        Err(_) => IoAreaDecode::Refused(side),
    }
}

/// Parse the `port_encoding` property value into entries keyed by base64 area id.
pub fn parse_port_encoding(
    value: &Value,
) -> Result<BTreeMap<String, PortEncoding>, FeagiDataError> {
    let object = value
        .as_object()
        .ok_or_else(|| bad("port_encoding must be a JSON object keyed by area id".to_string()))?;
    let mut entries = BTreeMap::new();
    for (area_id, entry) in object {
        CorticalID::try_from_base_64(area_id).map_err(|err| {
            bad(format!(
                "port_encoding key '{area_id}' is not a base64 cortical id: {err}"
            ))
        })?;
        entries.insert(area_id.clone(), PortEncoding::from_stored_value(entry)?);
    }
    Ok(entries)
}

/// Serialize entries to the `port_encoding` property value.
pub fn port_encoding_to_value(entries: &BTreeMap<String, PortEncoding>) -> Value {
    Value::Object(
        entries
            .iter()
            .map(|(area_id, entry)| (area_id.clone(), entry.to_value()))
            .collect(),
    )
}

/// The region's stored port encoding (empty when the property is absent).
pub fn region_port_encoding(
    region: &BrainRegion,
) -> Result<BTreeMap<String, PortEncoding>, FeagiDataError> {
    match region.properties.get(PORT_ENCODING_KEY) {
        Some(value) => parse_port_encoding(value),
        None => Ok(BTreeMap::new()),
    }
}

/// Write entries to the region; an empty map removes the property.
pub fn set_region_port_encoding(
    region: &mut BrainRegion,
    entries: &BTreeMap<String, PortEncoding>,
) {
    if entries.is_empty() {
        region.properties.remove(PORT_ENCODING_KEY);
    } else {
        region.properties.insert(
            PORT_ENCODING_KEY.to_string(),
            port_encoding_to_value(entries),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genomic::cortical_area::descriptors::{CorticalSubUnitIndex, CorticalUnitIndex};
    use crate::genomic::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, IOCorticalAreaConfigurationFlag as Flag, PercentageNeuronPositioning,
    };

    fn io_id(flag: Flag, is_input: bool) -> CorticalID {
        flag.as_io_cortical_id(
            is_input,
            *b"tst",
            CorticalUnitIndex::from(0u16),
            CorticalSubUnitIndex::from(0u8),
        )
    }

    const ABS_LINEAR: Flag = Flag::Percentage(
        FrameChangeHandling::Absolute,
        PercentageNeuronPositioning::Linear,
    );

    #[test]
    fn vocabulary_matches_the_flag_decoder_kinds() {
        // Every kind the flag decoder can emit is in the allowed vocabulary.
        for variant in 0u16..12 {
            let id = CorticalID::try_from_bytes(&[b'i', b't', b's', b't', variant as u8, 0, 0, 0])
                .unwrap();
            let IoAreaDecode::Fields(_, fields) = decode_io_area(&id) else {
                panic!("variant {variant} must decode");
            };
            assert!(PORT_KINDS.contains(&fields.kind), "{}", fields.kind);
        }
    }

    #[test]
    fn decode_gives_all_four_fields_for_percentage() {
        let id = io_id(ABS_LINEAR, true);
        let IoAreaDecode::Fields(side, fields) = decode_io_area(&id) else {
            panic!("must decode");
        };
        assert_eq!(side, IoPortSide::Input);
        let entry = PortEncoding::from_flag_fields(&fields);
        assert_eq!(entry.kind.as_deref(), Some("Percentage"));
        assert_eq!(entry.signed, Some(false));
        assert_eq!(entry.frame.as_deref(), Some("Absolute"));
        assert_eq!(entry.positioning.as_deref(), Some("Linear"));
        assert_eq!(entry.source, PortEncodingSource::Wiring);
    }

    #[test]
    fn decode_frame_only_kind_leaves_signed_and_positioning_null() {
        let id = io_id(Flag::Misc(FrameChangeHandling::Incremental), false);
        let IoAreaDecode::Fields(side, fields) = decode_io_area(&id) else {
            panic!("must decode");
        };
        assert_eq!(side, IoPortSide::Output);
        let entry = PortEncoding::from_flag_fields(&fields);
        assert_eq!(entry.kind.as_deref(), Some("Misc"));
        assert_eq!(entry.signed, None);
        assert_eq!(entry.frame.as_deref(), Some("Incremental"));
        assert_eq!(entry.positioning, None);
    }

    #[test]
    fn decode_refuses_positioning_bit_on_a_frame_only_kind_and_skips_non_io() {
        // variant 10 (Misc) with the positioning bit set: the flag decoder refuses it.
        let refused =
            CorticalID::try_from_bytes(&[b'i', b't', b's', b't', 0x0A, 0x02, 0, 0]).unwrap();
        assert_eq!(
            decode_io_area(&refused),
            IoAreaDecode::Refused(IoPortSide::Input)
        );
        let custom = CorticalID::try_from_bytes(b"cst_port").unwrap();
        assert_eq!(decode_io_area(&custom), IoAreaDecode::NotIo);
    }

    #[test]
    fn peers_disagreeing_on_one_field_null_only_that_field() {
        let absolute = ABS_LINEAR.port_encoding_fields();
        let incremental = Flag::Percentage(
            FrameChangeHandling::Incremental,
            PercentageNeuronPositioning::Linear,
        )
        .port_encoding_fields();
        let merged = PortEncoding::from_peer_fields(&[absolute, incremental]).unwrap();
        assert_eq!(merged.kind.as_deref(), Some("Percentage"));
        assert_eq!(merged.signed, Some(false));
        assert_eq!(merged.frame, None);
        assert_eq!(merged.positioning.as_deref(), Some("Linear"));
        assert!(PortEncoding::from_peer_fields(&[]).is_none());
    }

    #[test]
    fn stored_entry_round_trips_and_requires_a_source() {
        let entry = PortEncoding::from_flag_fields(&ABS_LINEAR.port_encoding_fields());
        let value = entry.to_value();
        assert_eq!(PortEncoding::from_stored_value(&value).unwrap(), entry);
        let mut without_source = value.clone();
        without_source.as_object_mut().unwrap().remove("source");
        assert!(PortEncoding::from_stored_value(&without_source).is_err());
    }

    #[test]
    fn vocabulary_violations_are_rejected() {
        for bad_value in [
            json!({"kind": "Nonsense", "source": "wiring"}),
            json!({"signed": "yes", "source": "wiring"}),
            json!({"frame": "Sideways", "source": "wiring"}),
            json!({"positioning": 3, "source": "wiring"}),
            json!({"kind": "Boolean", "source": "robot"}),
            json!({"kind": "Boolean", "source": "wiring", "extra": 1}),
            json!("Boolean"),
        ] {
            assert!(
                PortEncoding::from_stored_value(&bad_value).is_err(),
                "{bad_value}"
            );
        }
    }

    #[test]
    fn author_entry_rejects_a_source_key_and_treats_absent_as_null() {
        assert!(PortEncoding::from_author_value(&json!({"source": "wiring"})).is_err());
        let entry = PortEncoding::from_author_value(&json!({"kind": "Boolean"})).unwrap();
        assert_eq!(entry.kind.as_deref(), Some("Boolean"));
        assert_eq!(entry.signed, None);
        assert_eq!(entry.source, PortEncodingSource::Author);
    }

    #[test]
    fn map_parse_requires_base64_cortical_id_keys() {
        let id = CorticalID::try_from_bytes(b"cst_port").unwrap();
        let entry = PortEncoding::from_flag_fields(&ABS_LINEAR.port_encoding_fields());
        let good = json!({ id.as_base_64(): entry.to_value() });
        let parsed = parse_port_encoding(&good).unwrap();
        assert_eq!(parsed[&id.as_base_64()], entry);
        assert!(parse_port_encoding(&json!({"not base64!": entry.to_value()})).is_err());
        assert!(parse_port_encoding(&json!([])).is_err());
    }
}
