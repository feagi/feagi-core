// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Pattern-based connectivity - wildcard matching and transformations.

Supports absolute patterns (*, ?, !, int) and source-relative directional
patterns (?+, ?-, ?+=, ?-=, ?+N, ?-N, ?-N:?+M) for spatial connectivity.
*/

use crate::types::Position;

type Dimensions = (usize, usize, usize);

/// Direction along an axis relative to the source coordinate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Positive,
    Negative,
}

/// Pattern element types for specifying connectivity rules per axis.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternElement {
    /// `"*"` - matches any coordinate on this axis
    Wildcard,
    /// `"?"` - pass through source coordinate (dst = src)
    Skip,
    /// `"!"` - exclude source coordinate (all except src)
    Exclude,
    /// Absolute coordinate value
    Exact(i32),
    /// `"?+"` or `"?-"` - all coordinates strictly above/below src
    DirectionExclusive(Direction),
    /// `"?+="` or `"?-="` - all coordinates at or above/below src
    DirectionInclusive(Direction),
    /// `"?+N"` or `"?-N"` - single coordinate offset from src
    Offset(i32),
    /// `"?-A:?+B"` - inclusive range [src + lo, src + hi]
    Range(i32, i32),
}

impl PatternElement {
    /// Parse a pattern element from a string value.
    pub fn from_value(value: &str) -> Self {
        match value {
            "*" => PatternElement::Wildcard,
            "?" => PatternElement::Skip,
            "!" => PatternElement::Exclude,
            "?+" => PatternElement::DirectionExclusive(Direction::Positive),
            "?-" => PatternElement::DirectionExclusive(Direction::Negative),
            "?+=" => PatternElement::DirectionInclusive(Direction::Positive),
            "?-=" => PatternElement::DirectionInclusive(Direction::Negative),
            _ => {
                if let Some(range_str) = Self::try_parse_range(value) {
                    return range_str;
                }
                if let Some(offset) = Self::try_parse_offset(value) {
                    return offset;
                }
                if let Ok(num) = value.parse::<i32>() {
                    PatternElement::Exact(num)
                } else {
                    PatternElement::Wildcard
                }
            }
        }
    }

    /// Parse FFI integer encoding into a PatternElement.
    pub fn from_int(value: i32) -> Self {
        match value {
            -1 => PatternElement::Wildcard,
            -2 => PatternElement::Skip,
            -3 => PatternElement::Exclude,
            -10 => PatternElement::DirectionExclusive(Direction::Positive),
            -11 => PatternElement::DirectionExclusive(Direction::Negative),
            -12 => PatternElement::DirectionInclusive(Direction::Positive),
            -13 => PatternElement::DirectionInclusive(Direction::Negative),
            _ => PatternElement::Exact(value),
        }
    }

    /// Attempt to parse a relative range pattern like "?-1:?+1" or "?+2:?+5".
    /// Format: "?<sign><int>:?<sign><int>" where both bounds are relative to src.
    fn try_parse_range(value: &str) -> Option<PatternElement> {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let lo = Self::extract_relative_offset(parts[0])?;
        let hi = Self::extract_relative_offset(parts[1])?;
        Some(PatternElement::Range(lo, hi))
    }

    /// Attempt to parse a single offset pattern like "?+3" or "?-2".
    fn try_parse_offset(value: &str) -> Option<PatternElement> {
        let offset = Self::extract_relative_offset(value)?;
        Some(PatternElement::Offset(offset))
    }

    /// Extract a numeric offset from a "?+N" or "?-N" string.
    fn extract_relative_offset(s: &str) -> Option<i32> {
        if !s.starts_with('?') {
            return None;
        }
        let rest = &s[1..];
        if rest.is_empty() {
            return None;
        }
        if rest == "+" || rest == "-" || rest == "+=" || rest == "-=" {
            return None;
        }
        rest.parse::<i32>().ok()
    }
}

/// 3D pattern (x, y, z)
pub type Pattern3D = (PatternElement, PatternElement, PatternElement);

/// Match a coordinate against a pattern element (point-wise check).
pub fn match_pattern_element(element: &PatternElement, coordinate: i32, src_coord: i32) -> bool {
    match element {
        PatternElement::Wildcard => true,
        PatternElement::Skip => coordinate == src_coord,
        PatternElement::Exclude => coordinate != src_coord,
        PatternElement::Exact(val) => coordinate == *val,
        PatternElement::DirectionExclusive(Direction::Positive) => coordinate > src_coord,
        PatternElement::DirectionExclusive(Direction::Negative) => coordinate < src_coord,
        PatternElement::DirectionInclusive(Direction::Positive) => coordinate >= src_coord,
        PatternElement::DirectionInclusive(Direction::Negative) => coordinate <= src_coord,
        PatternElement::Offset(off) => coordinate == src_coord + off,
        PatternElement::Range(lo, hi) => {
            coordinate >= src_coord + lo && coordinate <= src_coord + hi
        }
    }
}

/// Expand a single axis pattern element into a set of destination coordinates.
fn expand_axis(element: &PatternElement, src_coord: u32, dim: usize) -> Vec<u32> {
    match element {
        PatternElement::Wildcard => (0..dim as u32).collect(),
        PatternElement::Skip => {
            if (src_coord as usize) < dim {
                vec![src_coord]
            } else {
                vec![]
            }
        }
        PatternElement::Exclude => (0..dim as u32).filter(|&c| c != src_coord).collect(),
        PatternElement::Exact(val) => {
            if *val >= 0 && (*val as usize) < dim {
                vec![*val as u32]
            } else {
                vec![]
            }
        }
        PatternElement::DirectionExclusive(Direction::Positive) => {
            ((src_coord + 1)..dim as u32).collect()
        }
        PatternElement::DirectionExclusive(Direction::Negative) => (0..src_coord).collect(),
        PatternElement::DirectionInclusive(Direction::Positive) => {
            (src_coord..dim as u32).collect()
        }
        PatternElement::DirectionInclusive(Direction::Negative) => {
            (0..=src_coord).filter(|&c| (c as usize) < dim).collect()
        }
        PatternElement::Offset(off) => {
            let target = src_coord as i32 + off;
            if target >= 0 && (target as usize) < dim {
                vec![target as u32]
            } else {
                vec![]
            }
        }
        PatternElement::Range(lo, hi) => {
            let start = (src_coord as i32 + lo).max(0) as u32;
            let end_exclusive = ((src_coord as i32 + hi) + 1).min(dim as i32) as u32;
            if start >= end_exclusive {
                vec![]
            } else {
                (start..end_exclusive).collect()
            }
        }
    }
}

/// Generate destination coordinates from pattern matching.
pub fn find_destination_coordinates(
    dst_dimensions: Dimensions,
    src_coordinate: Position,
    _src_pattern: &Pattern3D,
    dst_pattern: &Pattern3D,
) -> Vec<Position> {
    let (dst_width, dst_height, dst_depth) = dst_dimensions;
    let (src_x, src_y, src_z) = src_coordinate;

    let x_range = expand_axis(&dst_pattern.0, src_x, dst_width);
    let y_range = expand_axis(&dst_pattern.1, src_y, dst_height);
    let z_range = expand_axis(&dst_pattern.2, src_z, dst_depth);

    let mut results = Vec::with_capacity(x_range.len() * y_range.len() * z_range.len());
    for x in &x_range {
        for y in &y_range {
            for z in &z_range {
                results.push((*x, *y, *z));
            }
        }
    }

    results
}

/// Find source coordinates that match a pattern.
/// Directional/relative patterns are treated as wildcard on source side since
/// they require a specific source coordinate to resolve against.
pub fn find_source_coordinates(
    src_pattern: &Pattern3D,
    src_dimensions: Dimensions,
) -> Vec<Position> {
    let (src_width, src_height, src_depth) = src_dimensions;

    let x_range: Vec<u32> = match &src_pattern.0 {
        PatternElement::Wildcard => (0..src_width as u32).collect(),
        PatternElement::Exact(val) => {
            if *val >= 0 && (*val as usize) < src_width {
                vec![*val as u32]
            } else {
                vec![]
            }
        }
        _ => (0..src_width as u32).collect(),
    };

    let y_range: Vec<u32> = match &src_pattern.1 {
        PatternElement::Wildcard => (0..src_height as u32).collect(),
        PatternElement::Exact(val) => {
            if *val >= 0 && (*val as usize) < src_height {
                vec![*val as u32]
            } else {
                vec![]
            }
        }
        _ => (0..src_height as u32).collect(),
    };

    let z_range: Vec<u32> = match &src_pattern.2 {
        PatternElement::Wildcard => (0..src_depth as u32).collect(),
        PatternElement::Exact(val) => {
            if *val >= 0 && (*val as usize) < src_depth {
                vec![*val as u32]
            } else {
                vec![]
            }
        }
        _ => (0..src_depth as u32).collect(),
    };

    let mut results = Vec::with_capacity(x_range.len() * y_range.len() * z_range.len());
    for x in &x_range {
        for y in &y_range {
            for z in &z_range {
                results.push((*x, *y, *z));
            }
        }
    }

    results
}

/// Batch process pattern matching for multiple patterns.
pub fn match_patterns_batch(
    src_coordinate: Position,
    patterns: &[(Pattern3D, Pattern3D)],
    _src_dimensions: Dimensions,
    dst_dimensions: Dimensions,
) -> Vec<Position> {
    let mut all_results = Vec::new();

    for (src_pattern, dst_pattern) in patterns {
        let (src_x, src_y, src_z) = src_coordinate;

        let x_match = match &src_pattern.0 {
            PatternElement::Wildcard => true,
            PatternElement::Exact(val) => src_x == (*val as u32),
            _ => true,
        };

        let y_match = match &src_pattern.1 {
            PatternElement::Wildcard => true,
            PatternElement::Exact(val) => src_y == (*val as u32),
            _ => true,
        };

        let z_match = match &src_pattern.2 {
            PatternElement::Wildcard => true,
            PatternElement::Exact(val) => src_z == (*val as u32),
            _ => true,
        };

        if x_match && y_match && z_match {
            let mut results = find_destination_coordinates(
                dst_dimensions,
                src_coordinate,
                src_pattern,
                dst_pattern,
            );
            all_results.append(&mut results);
        }
    }

    all_results.sort_unstable();
    all_results.dedup();

    all_results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_pattern() {
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Exact(0),
        );
        let dst_pattern = (
            PatternElement::Skip,
            PatternElement::Skip,
            PatternElement::Exact(1),
        );

        let results =
            find_destination_coordinates((10, 10, 10), (5, 5, 0), &src_pattern, &dst_pattern);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], (5, 5, 1));
    }

    #[test]
    fn test_exact_pattern() {
        let src_pattern = (
            PatternElement::Exact(0),
            PatternElement::Exact(0),
            PatternElement::Exact(0),
        );
        let dst_pattern = (
            PatternElement::Exact(1),
            PatternElement::Exact(2),
            PatternElement::Exact(3),
        );

        let results =
            find_destination_coordinates((10, 10, 10), (0, 0, 0), &src_pattern, &dst_pattern);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], (1, 2, 3));
    }

    #[test]
    fn test_exclude_pattern() {
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );
        let dst_pattern = (
            PatternElement::Exclude,
            PatternElement::Exact(0),
            PatternElement::Exact(0),
        );

        let results =
            find_destination_coordinates((3, 1, 1), (1, 0, 0), &src_pattern, &dst_pattern);

        assert_eq!(results.len(), 2);
        assert!(results.contains(&(0, 0, 0)));
        assert!(results.contains(&(2, 0, 0)));
    }

    #[test]
    fn test_direction_positive_exclusive() {
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );
        let dst_pattern = (
            PatternElement::DirectionExclusive(Direction::Positive),
            PatternElement::Skip,
            PatternElement::Skip,
        );

        let results =
            find_destination_coordinates((8, 4, 2), (3, 1, 0), &src_pattern, &dst_pattern);

        assert_eq!(results, vec![(4, 1, 0), (5, 1, 0), (6, 1, 0), (7, 1, 0)]);
    }

    #[test]
    fn test_direction_negative_exclusive() {
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );
        let dst_pattern = (
            PatternElement::DirectionExclusive(Direction::Negative),
            PatternElement::Skip,
            PatternElement::Skip,
        );

        let results =
            find_destination_coordinates((8, 4, 2), (3, 1, 0), &src_pattern, &dst_pattern);

        assert_eq!(results, vec![(0, 1, 0), (1, 1, 0), (2, 1, 0)]);
    }

    #[test]
    fn test_direction_positive_inclusive() {
        let dst_pattern = (
            PatternElement::DirectionInclusive(Direction::Positive),
            PatternElement::Skip,
            PatternElement::Skip,
        );
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );

        let results =
            find_destination_coordinates((6, 3, 1), (2, 1, 0), &src_pattern, &dst_pattern);

        assert_eq!(results, vec![(2, 1, 0), (3, 1, 0), (4, 1, 0), (5, 1, 0)]);
    }

    #[test]
    fn test_direction_negative_inclusive() {
        let dst_pattern = (
            PatternElement::DirectionInclusive(Direction::Negative),
            PatternElement::Skip,
            PatternElement::Skip,
        );
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );

        let results =
            find_destination_coordinates((6, 3, 1), (2, 1, 0), &src_pattern, &dst_pattern);

        assert_eq!(results, vec![(0, 1, 0), (1, 1, 0), (2, 1, 0)]);
    }

    #[test]
    fn test_offset_positive() {
        let dst_pattern = (
            PatternElement::Offset(2),
            PatternElement::Skip,
            PatternElement::Skip,
        );
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );

        let results =
            find_destination_coordinates((10, 10, 10), (3, 5, 7), &src_pattern, &dst_pattern);

        assert_eq!(results, vec![(5, 5, 7)]);
    }

    #[test]
    fn test_offset_negative() {
        let dst_pattern = (
            PatternElement::Offset(-2),
            PatternElement::Skip,
            PatternElement::Skip,
        );
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );

        let results =
            find_destination_coordinates((10, 10, 10), (3, 5, 7), &src_pattern, &dst_pattern);

        assert_eq!(results, vec![(1, 5, 7)]);
    }

    #[test]
    fn test_offset_out_of_bounds() {
        let dst_pattern = (
            PatternElement::Offset(5),
            PatternElement::Skip,
            PatternElement::Skip,
        );
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );

        let results =
            find_destination_coordinates((6, 5, 5), (4, 2, 2), &src_pattern, &dst_pattern);

        assert!(results.is_empty());
    }

    #[test]
    fn test_range_symmetric() {
        let dst_pattern = (
            PatternElement::Range(-1, 1),
            PatternElement::Range(-1, 1),
            PatternElement::Skip,
        );
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );

        let results =
            find_destination_coordinates((10, 10, 5), (5, 5, 2), &src_pattern, &dst_pattern);

        assert_eq!(results.len(), 9); // 3x3 grid
        assert!(results.contains(&(4, 4, 2)));
        assert!(results.contains(&(5, 5, 2)));
        assert!(results.contains(&(6, 6, 2)));
    }

    #[test]
    fn test_range_clamped_at_boundary() {
        let dst_pattern = (
            PatternElement::Range(-3, 3),
            PatternElement::Skip,
            PatternElement::Skip,
        );
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );

        // src_x=1, range would be -2..4, clamped to 0..4
        let results =
            find_destination_coordinates((8, 1, 1), (1, 0, 0), &src_pattern, &dst_pattern);

        assert_eq!(
            results,
            vec![(0, 0, 0), (1, 0, 0), (2, 0, 0), (3, 0, 0), (4, 0, 0)]
        );
    }

    #[test]
    fn test_range_forward_only() {
        let dst_pattern = (
            PatternElement::Range(1, 3),
            PatternElement::Skip,
            PatternElement::Skip,
        );
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );

        let results =
            find_destination_coordinates((10, 1, 1), (2, 0, 0), &src_pattern, &dst_pattern);

        assert_eq!(results, vec![(3, 0, 0), (4, 0, 0), (5, 0, 0)]);
    }

    #[test]
    fn test_direction_at_edge() {
        let dst_pattern = (
            PatternElement::DirectionExclusive(Direction::Negative),
            PatternElement::Skip,
            PatternElement::Skip,
        );
        let src_pattern = (
            PatternElement::Wildcard,
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        );

        // src_x=0: nothing to the left
        let results =
            find_destination_coordinates((10, 1, 1), (0, 0, 0), &src_pattern, &dst_pattern);

        assert!(results.is_empty());
    }

    #[test]
    fn test_from_value_new_patterns() {
        assert_eq!(
            PatternElement::from_value("?+"),
            PatternElement::DirectionExclusive(Direction::Positive)
        );
        assert_eq!(
            PatternElement::from_value("?-"),
            PatternElement::DirectionExclusive(Direction::Negative)
        );
        assert_eq!(
            PatternElement::from_value("?+="),
            PatternElement::DirectionInclusive(Direction::Positive)
        );
        assert_eq!(
            PatternElement::from_value("?-="),
            PatternElement::DirectionInclusive(Direction::Negative)
        );
        assert_eq!(PatternElement::from_value("?+3"), PatternElement::Offset(3));
        assert_eq!(
            PatternElement::from_value("?-2"),
            PatternElement::Offset(-2)
        );
        assert_eq!(
            PatternElement::from_value("?-1:?+1"),
            PatternElement::Range(-1, 1)
        );
        assert_eq!(
            PatternElement::from_value("?+2:?+5"),
            PatternElement::Range(2, 5)
        );
    }

    #[test]
    fn test_from_value_backward_compat() {
        assert_eq!(PatternElement::from_value("*"), PatternElement::Wildcard);
        assert_eq!(PatternElement::from_value("?"), PatternElement::Skip);
        assert_eq!(PatternElement::from_value("!"), PatternElement::Exclude);
        assert_eq!(PatternElement::from_value("7"), PatternElement::Exact(7));
    }

    #[test]
    fn test_from_int_new_encodings() {
        assert_eq!(
            PatternElement::from_int(-10),
            PatternElement::DirectionExclusive(Direction::Positive)
        );
        assert_eq!(
            PatternElement::from_int(-11),
            PatternElement::DirectionExclusive(Direction::Negative)
        );
        assert_eq!(
            PatternElement::from_int(-12),
            PatternElement::DirectionInclusive(Direction::Positive)
        );
        assert_eq!(
            PatternElement::from_int(-13),
            PatternElement::DirectionInclusive(Direction::Negative)
        );
    }

    #[test]
    fn test_batch_with_directional() {
        let patterns = vec![(
            (
                PatternElement::Wildcard,
                PatternElement::Wildcard,
                PatternElement::Wildcard,
            ),
            (
                PatternElement::DirectionExclusive(Direction::Positive),
                PatternElement::Skip,
                PatternElement::Skip,
            ),
        )];

        let results = match_patterns_batch((3, 0, 0), &patterns, (8, 1, 1), (8, 1, 1));

        assert_eq!(results, vec![(4, 0, 0), (5, 0, 0), (6, 0, 0), (7, 0, 0)]);
    }
}
