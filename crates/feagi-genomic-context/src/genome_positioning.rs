// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Placement of structures within genome space.
//!
//! Genome space is where a genome says its cortical areas sit relative to one another, and it is
//! signed: areas are laid out around an origin, so coordinates run negative as readily as
//! positive. This is a distinct quantity from a voxel index, which addresses a neuron inside an
//! area and is therefore unsigned and bounded by that area's dimensions. Keeping the two in
//! separate types is what stops a placement from being silently truncated into a voxel index.

use feagi_data::values::spatial::integer_signed::SignedIntegerSpatial;

/// Quantization of a genome-space axis.
///
/// Genomes are authored by hand and by evolution at human scale, so `i32` covers the range with
/// room to spare while staying compact enough for the wire formats that carry it.
pub type GenomeAxisQuant = i32;

const GENOME_COORDINATE_DIMS: usize = 3;

/// Signed 3D coordinate in genome space.
pub type SignedCoordinate3D<Q> = SignedIntegerSpatial<Q, GENOME_COORDINATE_DIMS>;

/// Where a structure sits in genome space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenomeCoordinate3D(SignedCoordinate3D<GenomeAxisQuant>);

impl GenomeCoordinate3D {
    pub fn new(x: GenomeAxisQuant, y: GenomeAxisQuant, z: GenomeAxisQuant) -> Self {
        Self(SignedIntegerSpatial::new_from_array([x, y, z]))
    }

    /// The genome-space origin.
    pub fn origin() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn x(&self) -> GenomeAxisQuant {
        self.0.as_slice()[0]
    }

    pub fn y(&self) -> GenomeAxisQuant {
        self.0.as_slice()[1]
    }

    pub fn z(&self) -> GenomeAxisQuant {
        self.0.as_slice()[2]
    }

    /// The underlying quantized coordinate, for callers doing spatial math in `feagi-data`.
    pub fn as_signed_coordinate(&self) -> &SignedCoordinate3D<GenomeAxisQuant> {
        &self.0
    }
}

impl Default for GenomeCoordinate3D {
    fn default() -> Self {
        Self::origin()
    }
}

impl From<(GenomeAxisQuant, GenomeAxisQuant, GenomeAxisQuant)> for GenomeCoordinate3D {
    fn from((x, y, z): (GenomeAxisQuant, GenomeAxisQuant, GenomeAxisQuant)) -> Self {
        Self::new(x, y, z)
    }
}

impl From<GenomeCoordinate3D> for (GenomeAxisQuant, GenomeAxisQuant, GenomeAxisQuant) {
    fn from(coordinate: GenomeCoordinate3D) -> Self {
        (coordinate.x(), coordinate.y(), coordinate.z())
    }
}

impl From<SignedCoordinate3D<GenomeAxisQuant>> for GenomeCoordinate3D {
    fn from(coordinate: SignedCoordinate3D<GenomeAxisQuant>) -> Self {
        Self(coordinate)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for GenomeCoordinate3D {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Serialized as a bare triple so it round-trips through the genome's
        // `relative_coordinate` array without a wrapper object.
        (self.x(), self.y(), self.z()).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for GenomeCoordinate3D {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (x, y, z) = <(GenomeAxisQuant, GenomeAxisQuant, GenomeAxisQuant)>::deserialize(deserializer)?;
        Ok(Self::new(x, y, z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_axes_round_trip() {
        // The case that motivates this type: genomes place areas at negative coordinates.
        let position = GenomeCoordinate3D::new(20, 0, -20);

        assert_eq!(position.x(), 20);
        assert_eq!(position.y(), 0);
        assert_eq!(position.z(), -20);
    }

    #[test]
    fn converts_to_and_from_tuples() {
        let original = (-7, 13, -2);
        let position: GenomeCoordinate3D = original.into();

        assert_eq!(<(i32, i32, i32)>::from(position), original);
    }

    #[test]
    fn default_is_origin() {
        assert_eq!(GenomeCoordinate3D::default(), GenomeCoordinate3D::origin());
        assert_eq!(GenomeCoordinate3D::origin().z(), 0);
    }
}
