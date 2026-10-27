use crate::feagi_id::feagi_id::{FeagiID, HasBase64};



pub trait FeagiIDBase64<const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize>: FeagiID<ID_BYTE_COUNT> {

    /// Fails when the two length consts disagree.
    ///
    /// Padded URL-safe base64 uses 4 characters for every 3 source bytes, rounded up.
    const BASE64_LEN_MATCHES_SOURCE: () = assert!(
        BASE_64_BYTE_COUNT == (ID_BYTE_COUNT + 2) / 3 * 4,
        "BASE_64_BYTE_COUNT must be the padded URL-safe base64 length of ID_BYTE_COUNT"
    );

    type Original: HasBase64<ID_BYTE_COUNT, BASE_64_BYTE_COUNT, Base64Type=Self>;

    fn to_original(&self) -> Self::Original;

    fn from_original(original: &Self::Original) -> Self;

    // fn as_str (do not make copies, the bytes in here are already known to be utf8 safe)
    // fn try_from_str
}

// impl core::str::FromStr, TryFrom<&str>, TryFrom<&heapless::String, core::convert::AsRef<str>
// core::fmt::Display, crate::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement,
// crate::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement