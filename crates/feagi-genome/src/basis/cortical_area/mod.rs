pub mod foundation;
pub mod processing;

use crate::basis::cortical_area::foundation::{CTGeneratorFoundationSubTyped, CTStorageFoundationSubTyped, CTStorageOrGeneratorFoundationSubTyped, CorticalIdentifierFoundation, CorticalTypeFoundationSubtypeKind};
use crate::basis::cortical_area::processing::{CTGeneratorProcessingSubTyped, CTStorageOrGeneratorProcessingSubTyped, CTStorageProcessingSubTyped, CorticalIdentifierProcessing, CorticalTypeProcessingSubtypeKind};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

//region Cortical Type

/// A cortical type is a higher level description of a cortical area. It has a set layout, but
/// could be a variety of neuron layouts (of that given layout). This is first extended by another
/// trait for the specific cortical type, then implemented bya  struct for the cortical subtype
pub trait CorticalType<NM: NeuronModel<NeuronLayoutTag=Self::NeuronLayout>> {

    /// What neuron layout is this cortical type restricted to?
    type NeuronLayout: NeuronLayoutTag;

    /// What restrictions, if any, should be on the neuron layout?
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization>:
        NeuronLayoutRestrictor< // layouts must match
            FIQ, CorticalLevelContext=<<Self::NeuronLayout as NeuronLayoutTag>::Layout<FIQ> as NeuronLayoutData<FIQ>>::CorticalLevelContext
    >;

    /// What method of inner ID is used for this cortical type
    type IdentifierInner: CorticalAreaInnerID;

    /// The repr(u8) enum that represents the subtype under a given type
    type CorticalSubTypeKindEnum;

    /// Each cortical type has a unique byte identifier. This is used for cortical Identification
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind;

    /// Each cortical subtype has a unique byte identifier (amongst that type). This is used for 
    /// cortical Identification
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum;

    /// What regions is this cortical type allowed within the genome?
    const ALLOWED_REGIONS: CorticalAllowedRegions;

    //region Default impls

    // TODO Cortical Identifier Conversions

    //endregion
}

//endregion

//region Cortical Type Enums

/// A no-nested enum representing the type of cortical area
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalTypeKind{
    Foundation = 0,
    Processing = 1,
}

/// A nested enum representing the type and the subtype of a cortical area
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalTypeAndSubtype{
    Foundation(CorticalTypeFoundationSubtypeKind),
    Processing(CorticalTypeProcessingSubtypeKind),
}

pub enum CTGeneratorTyped<FIQ: FeagiIndexQuantization>
{
    Foundation(CTGeneratorFoundationSubTyped<FIQ>),
    Processing(CTGeneratorProcessingSubTyped<FIQ>)
}

pub enum CTStorageTyped<FIQ: FeagiIndexQuantization>
{
    Foundation(CTStorageFoundationSubTyped<FIQ>),
    Processing(CTStorageProcessingSubTyped<FIQ>)
}

pub enum CTStorageOrGeneratorTyped<FIQ: FeagiIndexQuantization>
{
    Foundation(CTStorageOrGeneratorFoundationSubTyped<FIQ>),
    Processing(CTStorageOrGeneratorProcessingSubTyped<FIQ>)
}

//region Impls

impl From<CorticalTypeAndSubtype> for CorticalTypeKind {
    fn from(value: CorticalTypeAndSubtype) -> Self {
        match value {
            CorticalTypeAndSubtype::Foundation(_) => CorticalTypeKind::Foundation,
            CorticalTypeAndSubtype::Processing(_) => CorticalTypeKind::Processing,
        }
    }
}

//endregion

//endregion

//region Cortical Identifier

#[repr(C, u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorticalIdentifier {
    Foundation(CorticalIdentifierFoundation) = 0,
    Processing(CorticalIdentifierProcessing) = 1,
}

impl CorticalIdentifier {

    fn try_from_bytes(bytes: [u8; 8]) -> Result<Self, CorticalIdentifierParseError> {
        match bytes[0] {
            0 => CorticalIdentifierFoundation::try_from_cortical_identifier_bytes(bytes),
            1 => CorticalIdentifierProcessing::try_from_cortical_identifier_bytes(bytes),
            err => Err(CorticalIdentifierParseError::InvalidCorticalType(err))
        }

    }

    /// Directly cast from bytes to this. Does NOT check validity!
    unsafe fn from_bytes_unsafe(bytes: [u8; 8]) -> Self {
        core::mem::transmute(bytes)
    }

    fn into_bytes(self) -> CorticalIdentifierBytes {
        const _: () = assert!(core::mem::size_of::<CorticalIdentifier>() == 8); // compile time check
        let arr: [u8; 8] = unsafe {core::mem::transmute(self)};
        CorticalIdentifierBytes(arr)
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CorticalIdentifierBytes(pub(crate) [u8; 8]);

impl CorticalIdentifierBytes {
    pub fn try_from_bytes(bytes: [u8; 8]) -> Result<Self, CorticalIdentifierParseError>
    {
        let decoded = CorticalIdentifier::try_from_bytes(bytes)?;
        Ok(decoded.into_bytes()) // 0 cost return
    }
    
    pub fn to_bytes(self) -> [u8; 8]
    {
        self.0
    }
    
    
}

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
pub trait CorticalAreaInnerID: Copy + Eq + core::hash::Hash + core::fmt::Debug + Into<[u8; 6]> + TryFrom<[u8; 6]> {}

/// just 6 bytes counts as well
impl CorticalAreaInnerID for [u8; 6] {}

//endregion

//region Faux Identifier


/// Used in place of the `CorticalIdentifier` in transformation / addition functions to eliminate
/// odds of collision while still having something mappings can match against
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct FauxCorticalIdentifier(u64);

impl FauxCorticalIdentifier {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

//endregion

//region Cortical Identifier Base 64
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use serde::{Deserialize, Serialize};
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


/// Allows specifying where a cortical area of the given type can reside in the genome
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalAllowedRegions {
    RootOnly,
    InnerOnly,
    RootOrInner, // look, its a bitset!
}