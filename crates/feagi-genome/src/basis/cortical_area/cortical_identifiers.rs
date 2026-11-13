use feagi_basis::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement;
use serde::{Deserialize, Serialize};
use base64::engine::general_purpose::URL_SAFE as BASE64_URL_SAFE;
use base64::Engine;
use feagi_basis::feagi_id::base_64::FeagiIDBase64Error;
use feagi_basis::feagi_message::feagi_message_description::FeagiFieldKind;
use feagi_basis::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement;
use crate::basis::cortical_area::foundation::CorticalIdentifierFoundation;
use crate::basis::cortical_area::processing::CorticalIdentifierProcessing;

// TODO manual byte serialization / deserialization to enforce 8 byte limits, note that serde
// ignores #[repr(C, u8)] overrides! So when bringing this down to 8 bytes we must do so manually

//region (Genuine) Cortical Identifier

/// The real counterpart to the `FauxCorticalIdentifier`
pub type GenuineCorticalIdentifier = CorticalIdentifier;

/// Identifies a cortical area within a genome, also known as a GenuineCorticalIdentifier as it
/// is the counterpart to a `FauxCorticalIdentifier`
#[repr(C, u8)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum CorticalIdentifier { // Due Faux, the max is 127
    Foundation(CorticalIdentifierFoundation) = 0,
    Processing(CorticalIdentifierProcessing) = 1,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CorticalIdentifierBytes(pub(crate) [u8; 8]);

impl CorticalIdentifierBytes {
    pub fn try_from_bytes(bytes: [u8; 8]) -> Result<Self, CorticalIdentifierParseError>
    {
        let decoded = Self::try_deserialize_from_bytes(bytes)?;
        Ok(Self::serialize_into_bytes(decoded)) // 0 cost return
    }

    pub fn to_bytes(self) -> [u8; 8]
    {
        self.0
    }

    fn try_deserialize_from_bytes(bytes: [u8; 8]) -> Result<CorticalIdentifier, CorticalIdentifierParseError> {
        match bytes[0] {
            0 => Ok(CorticalIdentifier::Foundation(CorticalIdentifierFoundation::try_from_cortical_identifier_bytes(bytes)?)),
            1 => Ok(CorticalIdentifier::Processing(CorticalIdentifierProcessing::try_from_cortical_identifier_bytes(bytes)?)),
            err => Err(CorticalIdentifierParseError::InvalidCorticalType(err))
        }
    }

    fn serialize_into_bytes(identifier: CorticalIdentifier) -> CorticalIdentifierBytes {
        const _: () = assert!(core::mem::size_of::<CorticalIdentifier>() == 8); // compile time check
        let arr: [u8; 8] = unsafe {core::mem::transmute(identifier)};
        CorticalIdentifierBytes(arr)
    }
}

//region Base 64

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
fn decode_identifier(text: &[u8]) -> Result<CorticalIdentifierBytes, CorticalIdentifierBase64Error> {
    if text.len() != TEXT_BYTE_COUNT {
        return Err(FeagiIDBase64Error::WrongLength { actual: text.len(), expected: TEXT_BYTE_COUNT });
    }
    let mut bytes = [0u8; 8];
    let decoded_length = BASE64_URL_SAFE.decode_slice(text, &mut bytes).map_err(|_| FeagiIDBase64Error::InvalidEncoding)?;
    // Text of the right length can still decode to fewer bytes, if it has more padding
    if decoded_length != bytes.len() {
        return Err(FeagiIDBase64Error::InvalidEncoding);
    }
    Ok(CorticalIdentifierBytes::try_from_bytes(bytes)?)
}

impl From<CorticalIdentifierBytes> for CorticalIdentifierBase64 {
    fn from(identifier: CorticalIdentifierBytes) -> Self {
        let mut text = [0u8; TEXT_BYTE_COUNT];
        match BASE64_URL_SAFE.encode_slice(identifier.to_bytes(), &mut text) {
            Ok(_) => Self(text),
            Err(_) => unreachable!("8 bytes always fit in 12 bytes of base64 text"),
        }
    }
}

impl TryFrom<CorticalIdentifierBase64> for CorticalIdentifierBytes {
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

//endregion

//endregion

//region Faux Cortical Identifier

/// Used in place of the `CorticalIdentifier` in transformation / addition functions to eliminate
/// odds of collision while still having something mappings can match against
#[repr(C, u8)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum FauxCorticalIdentifier { // Note: Offset of 128 to maintain difference with Genuine
    Foundation(CorticalIdentifierFoundation) = 128 + 0,
    Processing(CorticalIdentifierProcessing) = 128 + 1,
}

//endregion

//region Maybe Cortical Identifier

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)] // No serde explicitly, due be being 9 bytes
pub enum MaybeCorticalIdentifier {
    Genuine(CorticalIdentifier),
    Faux(FauxCorticalIdentifier)
}

//endregion

#[derive(Debug, thiserror::Error)]
pub enum CorticalIdentifierParseError {
    #[error("Decoded byte {0} corresponds to no known cortical type!")]
    InvalidCorticalType(u8),
    #[error("Decoded byte {0} corresponds to no known cortical subtype!")]
    InvalidCorticalSubtype(u8),
}

/// The last 6 bytes of a cortical ID, which each cortical type gives its own meaning. Converting to
/// bytes and back must return the exact same bytes, or two different byte patterns could mean the
/// same cortical area
pub trait CorticalAreaInnerID: Copy + Eq + core::hash::Hash + core::fmt::Debug + TryFrom<[u8; 6]> + Sized {
    
    /// Any additional data / systems needed to do the encoding. Is passed in as a mut reference to allow
    /// variety when retrying
    type AdditionalEncoderContext;
    
    /// What type of data is represented by the 6 bytes
    type RepresentingData;
    
    /// If the generation results in different IDs each time, we can retry up to this many times to
    /// try for different IDs in cases of collision.
    const NUMBER_RETRIES_ALLOWED: u8 = 0;
    
    fn encode_to_inner_id(data: Self::RepresentingData, additional_context: &mut Self::AdditionalEncoderContext) -> Self;
    
    fn from_inner_id_to_original_data(self) -> Self::RepresentingData;
    
    fn decode_from_bytes(bytes: &[u8; 6]) -> Result<Self, ()>;
}

/// An inner cortical id of 6 bytes that are randomly generated
#[derive(Clone, Copy, Hash, Debug, Eq, PartialEq)]
pub struct CorticalAreaInnerIDRandomBytes([u8; 6]);

impl TryFrom<[u8; 6]> for CorticalAreaInnerIDRandomBytes {
    type Error = ();

    fn try_from(value: [u8; 6]) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl CorticalAreaInnerID for CorticalAreaInnerIDRandomBytes {
    type AdditionalEncoderContext = (); // none needed
    type RepresentingData = (); // None, we just generate random

    fn encode_to_inner_id(data: Self::RepresentingData, additional_context: &mut Self::AdditionalEncoderContext) -> Self {
        todo!()
    }

    fn from_inner_id_to_original_data(self) -> Self::RepresentingData {
        () // nothing
    }

    fn decode_from_bytes(bytes: &[u8; 6]) -> Result<Self, ()> {
        Ok(Self(bytes.clone()))
    }
}



