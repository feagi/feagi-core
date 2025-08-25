//! Spatial data structures for FEAGI brain coordinates and dimensions.
//!
//! This module provides fundamental spatial primitives including 3D coordinates
//! and dimensional bounds checking for neural space representation.

use std::ops::Range;
use crate::FeagiDataError;

//region 2D Coordinates


//endregion

/// 2D coordinate with unsigned 32-bit integer components.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Copy)]
pub struct FlatCoordinateU32 {
    pub x: u32,
    pub y: u32,
}

impl FlatCoordinateU32 {
    /// Creates a new 2D coordinate.
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl std::fmt::Display for FlatCoordinateU32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}


//region 3D Coordinates

/// 3D coordinate with unsigned 32-bit integer components.
/// Used for representing positions in neural space.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct CoordinateU32 {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl CoordinateU32 {
    /// Creates a new 3D coordinate.
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
}

impl std::fmt::Display for CoordinateU32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}


/// 3D coordinate with signed 32-bit integer components.
/// Used for representing relative positions or offsets in neural space.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct CoordinateI32 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl CoordinateI32 {
    /// Creates a new 3D signed coordinate.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl std::fmt::Display for CoordinateI32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}
// TODO try from for I and U 32

//endregion



//region 2D Dimensions

/// Represents the resolution of a 2D array like object (such as an image (ignoring color channels))
/// Cannot be 0 in any direction
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct CartesianResolution {
    pub width: usize,
    pub height: usize,
}

impl std::fmt::Display for CartesianResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{}, {}>", self.width, self.height)
    }
}

impl CartesianResolution {
    pub fn new(width: usize, height: usize) -> Result<Self, FeagiDataError> {
        if width == 0 || height == 0 {
            return Err(FeagiDataError::BadParameters("Width or Height cannot be 0!".to_string()));
        }
        Ok(Self { width, height })
    }
}

//endregion



//region 3D Dimensions
/// 3D dimensions defining the bounds of neural space.
/// All dimensions must be non-zero.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Dimensions{
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Dimensions{
    /// Creates new dimensions, ensuring all values are non-zero.
    pub fn new(x: u32, y: u32, z: u32) -> Result<Self, FeagiDataError> {
        if x == 0 || y == 0 || z == 0{
            return Err(FeagiDataError::BadParameters("Dimensions in any direction cannot be zero!".into()))
        }
        Ok(Dimensions{x, y, z})
    }
    
    /// Verifies that a coordinate falls within these dimensional bounds.
    pub fn verify_coordinate_in_bounds(&self, positive_coordinates: &CoordinateU32) -> Result<(), FeagiDataError>{
        if  positive_coordinates.x < self.x && positive_coordinates.y < self.y && positive_coordinates.z < self.z{
            return Ok(())
        }
        Err(FeagiDataError::BadParameters(format!("Point {} is not within the dimension bounds of {}!", positive_coordinates, self)))
    }
}

impl std::fmt::Display for Dimensions{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{}, {}, {}>", self.x, self.y, self.z)
    }
}



/// A 3D range defining acceptable coordinate bounds.
/// Each axis has its own range for flexible boundary definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimensionRange{
    pub x: Range<u32>,
    pub y: Range<u32>,
    pub z: Range<u32>,
}

impl DimensionRange {
    /// Creates a new dimension range, ensuring no ranges are empty.
    pub fn new(x: Range<u32>, y: Range<u32>, z: Range<u32>) -> Result<DimensionRange, FeagiDataError> {
        if x.is_empty() || y.is_empty() || z.is_empty() {
            return Err(FeagiDataError::BadParameters("A given range has some empty or invalid ranges!".into()))
        }
        Ok(DimensionRange { x, y, z })
    }
    
    /// Returns true if any axis spans more than one value (i.e., not a single point).
    pub fn is_ambiguous(&self) -> bool {
        self.x.len() != 1 || self.y.len() != 1 || self.z.len() != 1
    }
    
    /// Verifies that a coordinate falls within all axis ranges.
    pub fn verify_coordinate_u32_within_range(&self, checking: &CoordinateU32) -> Result<(), FeagiDataError> {
        if !self.x.contains(&checking.x) || !self.y.contains(&checking.y) || !self.z.contains(&checking.z){
            return Err(FeagiDataError::BadParameters(format!("Point {} is not within the acceptable range of {}!", checking, self)));
        }
        Ok(())

    }
}

impl std::fmt::Display for DimensionRange{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{} - {}, {} - {}, {} - {}>", self.x.start, self.x.end, self.y.start, self.y.end, self.z.start, self.z.end)
    }
}

/*
——————————No Macros?——————————
⠀⣞⢽⢪⢣⢣⢣⢫⡺⡵⣝⡮⣗⢷⢽⢽⢽⣮⡷⡽⣜⣜⢮⢺⣜⢷⢽⢝⡽⣝
⠸⡸⠜⠕⠕⠁⢁⢇⢏⢽⢺⣪⡳⡝⣎⣏⢯⢞⡿⣟⣷⣳⢯⡷⣽⢽⢯⣳⣫⠇
⠀⠀⢀⢀⢄⢬⢪⡪⡎⣆⡈⠚⠜⠕⠇⠗⠝⢕⢯⢫⣞⣯⣿⣻⡽⣏⢗⣗⠏⠀
⠀⠪⡪⡪⣪⢪⢺⢸⢢⢓⢆⢤⢀⠀⠀⠀⠀⠈⢊⢞⡾⣿⡯⣏⢮⠷⠁⠀⠀
⠀⠀⠀⠈⠊⠆⡃⠕⢕⢇⢇⢇⢇⢇⢏⢎⢎⢆⢄⠀⢑⣽⣿⢝⠲⠉⠀⠀⠀⠀
⠀⠀⠀⠀⠀⡿⠂⠠⠀⡇⢇⠕⢈⣀⠀⠁⠡⠣⡣⡫⣂⣿⠯⢪⠰⠂⠀⠀⠀⠀
⠀⠀⠀⠀⡦⡙⡂⢀⢤⢣⠣⡈⣾⡃⠠⠄⠀⡄⢱⣌⣶⢏⢊⠂⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢝⡲⣜⡮⡏⢎⢌⢂⠙⠢⠐⢀⢘⢵⣽⣿⡿⠁⠁⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠨⣺⡺⡕⡕⡱⡑⡆⡕⡅⡕⡜⡼⢽⡻⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣼⣳⣫⣾⣵⣗⡵⡱⡡⢣⢑⢕⢜⢕⡝⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⣴⣿⣾⣿⣿⣿⡿⡽⡑⢌⠪⡢⡣⣣⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⡟⡾⣿⢿⢿⢵⣽⣾⣼⣘⢸⢸⣞⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠁⠇⠡⠩⡫⢿⣝⡻⡮⣒⢽⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
—————————————————————————————
 */

//endregion