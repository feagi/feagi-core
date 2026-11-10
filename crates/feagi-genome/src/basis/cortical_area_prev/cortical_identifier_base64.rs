//! The form of a [`CorticalIdentifier`] used in paths and anywhere else text is needed

use super::{CorticalIdentifier, CorticalIdentifierParseError};
use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use feagi_basis::feagi_id::base_64::FeagiIDBase64Error;
use feagi_basis::feagi_message::feagi_message_description::FeagiFieldKind;
use feagi_basis::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement;
use feagi_basis::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement;

/// How many bytes a [`CorticalIdentifier`] takes as padded base64 text
const TEXT_BYTE_COUNT: usize = 12;

/// Failure to parse a [`CorticalIdentifierBase64`] from text
pub type CorticalIdentifierBase64Error = FeagiIDBase64Error<CorticalIdentifierParseError>;

/// The padded URL-safe base64 text of a [`CorticalIdentifier`]. The field is private so that only
/// the canonical text of a valid identifier can be held
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorticalIdentifierBase64([u8; TEXT_BYTE_COUNT]);

impl CorticalIdentifierBase64 {
    /// The base64 text
    pub fn as_str(&self) -> &str {
        match core::str::from_utf8(&self.0) {
            Ok(text) => text,
            Err(_) => unreachable!("base64 text is always ASCII"),
        }
    }

    /// Parses canonical padded URL-safe base64 text of a valid identifier
    pub fn try_from_str(text: &str) -> Result<Self, CorticalIdentifierBase64Error> {
        Ok(Self::from(decode_identifier(text.as_bytes())?))
    }
}

/// Decodes the text into the bytes of an identifier, which must be exactly the length of one, and
/// then validates them
fn decode_identifier(text: &[u8]) -> Result<CorticalIdentifier, CorticalIdentifierBase64Error> {
    if text.len() != TEXT_BYTE_COUNT {
        return Err(FeagiIDBase64Error::WrongLength { actual: text.len(), expected: TEXT_BYTE_COUNT });
    }
    let mut bytes = [0u8; 8];
    let decoded_length = BASE64_URL_SAFE.decode_slice(text, &mut bytes).map_err(|_| FeagiIDBase64Error::InvalidEncoding)?;
    // Text of the right length can still decode to fewer bytes, if it has more padding
    if decoded_length != bytes.len() {
        return Err(FeagiIDBase64Error::InvalidEncoding);
    }
    Ok(CorticalIdentifier::try_from_bytes(bytes)?)
}

impl From<CorticalIdentifier> for CorticalIdentifierBase64 {
    fn from(identifier: CorticalIdentifier) -> Self {
        let mut text = [0u8; TEXT_BYTE_COUNT];
        match BASE64_URL_SAFE.encode_slice(identifier.to_bytes(), &mut text) {
            Ok(_) => Self(text),
            Err(_) => unreachable!("8 bytes always fit in 12 bytes of base64 text"),
        }
    }
}

impl TryFrom<CorticalIdentifierBase64> for CorticalIdentifier {
    type Error = CorticalIdentifierBase64Error;

    fn try_from(base64: CorticalIdentifierBase64) -> Result<Self, Self::Error> {
        decode_identifier(&base64.0)
    }
}

impl core::str::FromStr for CorticalIdentifierBase64 {
    type Err = CorticalIdentifierBase64Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(text)
    }
}

impl<'a> TryFrom<&'a str> for CorticalIdentifierBase64 {
    type Error = CorticalIdentifierBase64Error;

    fn try_from(text: &'a str) -> Result<Self, Self::Error> {
        Self::try_from_str(text)
    }
}

impl AsRef<str> for CorticalIdentifierBase64 {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl core::fmt::Display for CorticalIdentifierBase64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FeagiMessagePathParameterElement for CorticalIdentifierBase64 {
    const KIND: FeagiFieldKind = FeagiFieldKind::String;

    fn write_path_segment<W: core::fmt::Write>(&self, out: &mut W) -> core::fmt::Result {
        out.write_str(self.as_str())
    }
}

impl FeagiMessagePathStaticElement for CorticalIdentifierBase64 {
    fn to_str_slice(&self) -> &str {
        self.as_str()
    }
}

impl serde::Serialize for CorticalIdentifierBase64 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for CorticalIdentifierBase64 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TextVisitor;

        impl serde::de::Visitor<'_> for TextVisitor {
            type Value = CorticalIdentifierBase64;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("a padded URL-safe base64 cortical identifier")
            }

            // Borrowed and owned strings both forward here by default
            fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<Self::Value, E> {
                CorticalIdentifierBase64::try_from_str(text).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(TextVisitor)
    }
}
