use crate::data_types::descriptors::MiscDataDimensions;
use crate::data_types::MiscData;
use feagi_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
use feagi_structures::FeagiDataError;

/// A single token ID transported through FEAGI as a Z-bitplane value at (x=0,y=0).
///
/// Encoding contract (shared across clients):
/// - One token per FEAGI tick.
/// - Coordinates are fixed at x=0, y=0.
/// - Bit index is encoded along z (z=0 is MSB).
/// - Active bit is represented by any potential `p > 0.0`.
/// - **Gap semantics**: absence of any active bitplanes means "no token emitted".
/// - **Offset encoding**: the represented integer is `token_id + 1` so that the
///   all-zero frame can be reserved for gaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextToken {
    token_id: u32,
}

impl TextToken {
    #[inline]
    pub const fn token_id(&self) -> u32 {
        self.token_id
    }

    #[inline]
    pub const fn new_unchecked(token_id: u32) -> Self {
        Self { token_id }
    }

    /// Decode a token from XYZP bitplanes.
    ///
    /// Returns:
    /// - `Ok(None)` for a gap (no token emitted).
    /// - `Ok(Some(TextToken))` for a decoded token id.
    pub fn try_from_xyzp_bitplanes(
        voxels: &NeuronVoxelXYZPArrays,
        depth: u32,
    ) -> Result<Option<Self>, FeagiDataError> {
        let Some(token_id) = decode_token_id_from_xyzp_bitplanes(voxels, depth)? else {
            return Ok(None);
        };
        Ok(Some(Self { token_id }))
    }

    /// Encode this token into XYZP bitplanes at (x=0,y=0) with z=0 as MSB.
    pub fn to_xyzp_bitplanes(&self, depth: u32) -> Result<NeuronVoxelXYZPArrays, FeagiDataError> {
        encode_token_id_to_xyzp_bitplanes(self.token_id, depth)
    }
}

/// Decode a token id from XYZP bitplanes (see [`TextToken`] for encoding contract).
pub fn decode_token_id_from_xyzp_bitplanes(
    voxels: &NeuronVoxelXYZPArrays,
    depth: u32,
) -> Result<Option<u32>, FeagiDataError> {
    if depth == 0 {
        return Err(FeagiDataError::BadParameters(
            "TextToken depth must be > 0".into(),
        ));
    }
    if depth > 32 {
        return Err(FeagiDataError::BadParameters(
            "TextToken depth must be <= 32 (u32 bit-width)".into(),
        ));
    }

    let (x, y, z, p) = voxels.borrow_xyzp_vectors();
    let mut value: u32 = 0;
    for i in 0..x.len() {
        if x[i] != 0 || y[i] != 0 {
            continue;
        }
        let zi = z[i];
        if zi >= depth {
            continue;
        }
        if p[i] <= 0.0 {
            continue;
        }
        let weight = 1u32 << (depth - 1 - zi);
        value |= weight;
    }

    if value == 0 {
        return Ok(None);
    }

    // Offset encoding: represented value is token_id+1.
    Ok(Some(value - 1))
}

/// Encode a token id into XYZP bitplanes (see [`TextToken`] for encoding contract).
pub fn encode_token_id_to_xyzp_bitplanes(
    token_id: u32,
    depth: u32,
) -> Result<NeuronVoxelXYZPArrays, FeagiDataError> {
    if depth == 0 {
        return Err(FeagiDataError::BadParameters(
            "TextToken depth must be > 0".into(),
        ));
    }
    if depth > 32 {
        return Err(FeagiDataError::BadParameters(
            "TextToken depth must be <= 32 (u32 bit-width)".into(),
        ));
    }

    // Offset encoding: store token_id+1 so value=0 is reserved for gaps.
    let value = token_id
        .checked_add(1)
        .ok_or_else(|| FeagiDataError::BadParameters("TextToken token_id overflow".into()))?;

    // Ensure representable with the provided bit depth.
    if depth < 32 && value >= (1u32 << depth) {
        return Err(FeagiDataError::BadParameters(format!(
            "TextToken token_id out of range for depth={depth} (value={value})"
        )));
    }

    let mut x_vec: Vec<u32> = Vec::new();
    let mut y_vec: Vec<u32> = Vec::new();
    let mut z_vec: Vec<u32> = Vec::new();
    let mut p_vec: Vec<f32> = Vec::new();

    for zi in 0..depth {
        let mask = 1u32 << (depth - 1 - zi);
        if (value & mask) != 0 {
            x_vec.push(0);
            y_vec.push(0);
            z_vec.push(zi);
            // Match the established sensory encoding convention used by image inputs:
            // potentials are carried in byte-intensity scale (0..255) and are compared
            // against neuron thresholds inside FEAGI.
            //
            // Decoders treat any p>0 as an active bit, so this remains compatible.
            p_vec.push(255.0);
        }
    }

    NeuronVoxelXYZPArrays::new_from_vectors(x_vec, y_vec, z_vec, p_vec)
}

/// Decode a token id from a 1x1xZ `MiscData` bitplane buffer (see [`TextToken`] contract).
pub fn decode_token_id_from_misc_data(misc: &MiscData) -> Result<Option<u32>, FeagiDataError> {
    let dims = misc.get_dimensions();
    if dims.width != 1 || dims.height != 1 {
        return Err(FeagiDataError::BadParameters(format!(
            "TextToken MiscData must be 1x1xdepth, got {}x{}x{}",
            dims.width, dims.height, dims.depth
        )));
    }
    decode_token_id_from_misc_data_with_depth(misc, dims.depth)
}

/// Decode a token id from a `MiscData` bitplane buffer, validating against an explicit depth.
pub fn decode_token_id_from_misc_data_with_depth(
    misc: &MiscData,
    depth: u32,
) -> Result<Option<u32>, FeagiDataError> {
    if depth == 0 {
        return Err(FeagiDataError::BadParameters(
            "TextToken depth must be > 0".into(),
        ));
    }
    if depth > 32 {
        return Err(FeagiDataError::BadParameters(
            "TextToken depth must be <= 32 (u32 bit-width)".into(),
        ));
    }

    let data = misc.get_internal_data();
    let mut value: u32 = 0;
    for zi in 0..depth {
        let v = data[(0, 0, zi as usize)];
        if v > 0.0 {
            let weight = 1u32 << (depth - 1 - zi);
            value |= weight;
        }
    }

    if value == 0 {
        return Ok(None);
    }
    Ok(Some(value - 1))
}

/// Encode a token id into a 1x1x`depth` `MiscData` bitplane buffer (see [`TextToken`] contract).
pub fn encode_token_id_to_misc_data(token_id: u32, depth: u32) -> Result<MiscData, FeagiDataError> {
    if depth == 0 {
        return Err(FeagiDataError::BadParameters(
            "TextToken depth must be > 0".into(),
        ));
    }
    if depth > 32 {
        return Err(FeagiDataError::BadParameters(
            "TextToken depth must be <= 32 (u32 bit-width)".into(),
        ));
    }

    // Validate representability (same rule as XYZP codec).
    let value = token_id
        .checked_add(1)
        .ok_or_else(|| FeagiDataError::BadParameters("TextToken token_id overflow".into()))?;
    if depth < 32 && value >= (1u32 << depth) {
        return Err(FeagiDataError::BadParameters(format!(
            "TextToken token_id out of range for depth={depth} (value={value})"
        )));
    }

    let dims = MiscDataDimensions::new(1, 1, depth)?;
    let mut misc = MiscData::new(&dims)?;
    let data = misc.get_internal_data_mut();

    for zi in 0..depth {
        let mask = 1u32 << (depth - 1 - zi);
        if (value & mask) != 0 {
            data[(0, 0, zi as usize)] = 1.0;
        }
    }

    Ok(misc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_decodes_to_none() {
        let voxels = NeuronVoxelXYZPArrays::new();
        let decoded = decode_token_id_from_xyzp_bitplanes(&voxels, 16).unwrap();
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_roundtrip_token_id_depth_16() {
        let token_id = 42u32;
        let voxels = encode_token_id_to_xyzp_bitplanes(token_id, 16).unwrap();
        let decoded = decode_token_id_from_xyzp_bitplanes(&voxels, 16).unwrap();
        assert_eq!(decoded, Some(token_id));
    }

    #[test]
    fn test_msb_is_z0() {
        // depth=4 -> MSB mask is 1<<3
        // Use token_id=7 -> value=8 (1000b) should set only z=0.
        let token_id = 7u32;
        let voxels = encode_token_id_to_xyzp_bitplanes(token_id, 4).unwrap();
        let (_x, _y, z, _p) = voxels.borrow_xyzp_vectors();
        assert_eq!(z, &vec![0u32]);
    }

    #[test]
    fn test_roundtrip_misc_data_depth_16() {
        let token_id = 50_256u32;
        let misc = encode_token_id_to_misc_data(token_id, 16).unwrap();
        let decoded = decode_token_id_from_misc_data(&misc).unwrap();
        assert_eq!(decoded, Some(token_id));
    }
}
