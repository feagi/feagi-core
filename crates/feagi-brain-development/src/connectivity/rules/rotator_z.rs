// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Z-axis rotation rule for function morphology `rotator_z`.

Maps one source XY voxel into every destination Z layer, where each layer
represents an incremental rotation in [-90, +90] degrees around the XY center.
*/

use crate::types::{BduError, BduResult, Position};

/// Map one source position to all destination z-layers using incremental XY rotation.
///
/// - Destination z=0 maps to -90 degrees (counter-clockwise).
/// - Destination z=depth-1 maps to +90 degrees (clockwise).
/// - Middle layer(s) are near 0 degrees.
///
/// Coordinates that rotate out-of-bounds are dropped for that layer.
pub fn syn_rotator_z(
    src_position: Position,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
) -> BduResult<Vec<Position>> {
    let (src_w, src_h, _) = src_dimensions;
    let (dst_w, dst_h, dst_d) = dst_dimensions;

    if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 || dst_d == 0 {
        return Ok(Vec::new());
    }

    let (src_x, src_y, _) = src_position;
    if src_x as usize >= src_w || src_y as usize >= src_h {
        return Err(BduError::OutOfBounds {
            pos: src_position,
            dims: src_dimensions,
        });
    }

    // Normalize source coordinates into [0, 1] based on source extents so we can
    // handle source/destination XY dimension mismatches deterministically.
    let src_w_span = (src_w.saturating_sub(1)).max(1) as f64;
    let src_h_span = (src_h.saturating_sub(1)).max(1) as f64;
    let norm_x = src_x as f64 / src_w_span;
    let norm_y = src_y as f64 / src_h_span;

    let dst_w_span = (dst_w.saturating_sub(1)).max(1) as f64;
    let dst_h_span = (dst_h.saturating_sub(1)).max(1) as f64;

    let base_x = norm_x * dst_w_span;
    let base_y = norm_y * dst_h_span;

    let center_x = dst_w_span / 2.0;
    let center_y = dst_h_span / 2.0;

    let mut out = Vec::with_capacity(dst_d);
    for z in 0..dst_d {
        let angle_deg = layer_angle_degrees(z, dst_d);
        let angle_rad = angle_deg.to_radians();

        // "Clockwise positive" convention per requirement:
        // x' = cx + cos(t)*(x-cx) + sin(t)*(y-cy)
        // y' = cy - sin(t)*(x-cx) + cos(t)*(y-cy)
        let dx = base_x - center_x;
        let dy = base_y - center_y;
        let rotated_x = center_x + angle_rad.cos() * dx + angle_rad.sin() * dy;
        let rotated_y = center_y - angle_rad.sin() * dx + angle_rad.cos() * dy;

        let rx = rotated_x.round();
        let ry = rotated_y.round();

        if rx >= 0.0 && rx < dst_w as f64 && ry >= 0.0 && ry < dst_h as f64 {
            out.push((rx as u32, ry as u32, z as u32));
        }
    }

    Ok(out)
}

#[inline]
fn layer_angle_degrees(z_index: usize, depth: usize) -> f64 {
    if depth <= 1 {
        return 0.0;
    }
    -90.0 + 180.0 * (z_index as f64 / (depth - 1) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_three_angles_match_endpoints_and_center() {
        assert!((layer_angle_degrees(0, 3) + 90.0).abs() < 1e-9);
        assert!(layer_angle_degrees(1, 3).abs() < 1e-9);
        assert!((layer_angle_degrees(2, 3) - 90.0).abs() < 1e-9);
    }

    #[test]
    fn test_depth_one_is_identity() {
        assert!(layer_angle_degrees(0, 1).abs() < 1e-9);
    }

    #[test]
    fn test_rotator_returns_one_candidate_per_layer_when_in_bounds() {
        let mapped = syn_rotator_z((16, 16, 0), (32, 32, 1), (32, 32, 3)).unwrap();
        assert_eq!(mapped.len(), 3);
        assert_eq!(mapped[0].2, 0);
        assert_eq!(mapped[1].2, 1);
        assert_eq!(mapped[2].2, 2);
    }

    #[test]
    fn test_rotator_reports_out_of_bounds_source() {
        let err = syn_rotator_z((100, 0, 0), (32, 32, 1), (32, 32, 10)).unwrap_err();
        match err {
            BduError::OutOfBounds { .. } => {}
            _ => panic!("Expected OutOfBounds"),
        }
    }
}
