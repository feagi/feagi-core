// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! One shared placement rule for the cortical areas an agent registration creates.
//!
//! Sensor and motor units use the same rule, so unit N of any type sits in its own row:
//!
//! - Y: `template_y + unit_index * stride`, where
//!   `stride = max(MIN_UNIT_STRIDE_Y, tallest template sub-area height + gap)`.
//! - Z: the template z.
//! - X: the first sub-area sits at its template x. Later sub-areas of a unit chain from the
//!   previous one (plus a gap), but only when every sub-area of the type shares the template
//!   y and z, and then in the direction the template x values run (leftward when they fall
//!   with the sub-area index, rightward when they rise). Types whose sub-areas are laid out
//!   in a grid keep their template x.
//!
//! Only areas that do not exist yet are placed; registration never moves existing areas.

/// Smallest vertical distance between two units of one type.
pub(crate) const MIN_UNIT_STRIDE_Y: i32 = 20;
/// Empty voxels kept between neighbouring areas (chain gap and vertical gap).
pub(crate) const AREA_GAP_VOXELS: i32 = 10;

/// What the placement rule needs to know about one sub-area template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SubAreaTemplate {
    pub position: [i32; 3],
    /// Template (default) height of the sub-area in voxels.
    pub height: i32,
}

/// Vertical distance between unit `N` and unit `N + 1` of a type.
pub(crate) fn unit_stride_y(templates: &[Option<SubAreaTemplate>]) -> i32 {
    let tallest = templates
        .iter()
        .flatten()
        .map(|t| t.height.max(1))
        .max()
        .unwrap_or(1);
    MIN_UNIT_STRIDE_Y.max(tallest.saturating_add(AREA_GAP_VOXELS))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChainDirection {
    Left,
    Right,
}

fn chain_direction(templates: &[Option<SubAreaTemplate>]) -> Option<ChainDirection> {
    let present: Vec<&SubAreaTemplate> = templates.iter().flatten().collect();
    if present.len() < 2 {
        return None;
    }
    let (y, z) = (present[0].position[1], present[0].position[2]);
    if present
        .iter()
        .any(|t| t.position[1] != y || t.position[2] != z)
    {
        return None;
    }
    if present
        .windows(2)
        .all(|w| w[1].position[0] < w[0].position[0])
    {
        Some(ChainDirection::Left)
    } else if present
        .windows(2)
        .all(|w| w[1].position[0] > w[0].position[0])
    {
        Some(ChainDirection::Right)
    } else {
        None
    }
}

/// World position of every sub-area of unit `unit_index`.
///
/// `templates[i]` and `widths[i]` describe sub-area `i` (`None` template: no such sub-area).
/// `stack_in_y` is false for types whose units already separate in X by their own rule.
pub(crate) fn unit_sub_area_positions(
    unit_index: u16,
    templates: &[Option<SubAreaTemplate>],
    widths: &[i32],
    stack_in_y: bool,
) -> Vec<Option<(i32, i32, i32)>> {
    let stride = unit_stride_y(templates);
    let direction = chain_direction(templates);
    let mut previous: Option<(i32, i32)> = None; // (x, width)
    let mut out = vec![None; templates.len()];
    for (i, template) in templates.iter().enumerate() {
        let Some(template) = template else {
            continue;
        };
        let width = widths.get(i).copied().unwrap_or(1).max(1);
        let x = match (direction, previous) {
            (Some(ChainDirection::Left), Some((px, _))) => px - width - AREA_GAP_VOXELS,
            (Some(ChainDirection::Right), Some((px, pw))) => px + pw + AREA_GAP_VOXELS,
            _ => template.position[0],
        };
        let y = if stack_in_y {
            template.position[1] + (unit_index as i32) * stride
        } else {
            template.position[1]
        };
        out[i] = Some((x, y, template.position[2]));
        previous = Some((x, width));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::cortical_area::descriptors::CorticalSubUnitIndex;
    use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit, UnitTopology};
    use std::collections::HashMap;

    type Box3 = ((i32, i32, i32), (i32, i32, i32)); // min, exclusive max

    fn templates_of(
        topology: &HashMap<CorticalSubUnitIndex, UnitTopology>,
    ) -> Vec<Option<SubAreaTemplate>> {
        let n = topology.len();
        (0..n)
            .map(|i| {
                topology
                    .get(&CorticalSubUnitIndex::from(i as u8))
                    .map(|t| SubAreaTemplate {
                        position: t.relative_position,
                        height: t.channel_dimensions_default[1] as i32,
                    })
            })
            .collect()
    }

    fn default_dims(topology: &HashMap<CorticalSubUnitIndex, UnitTopology>) -> Vec<i32> {
        (0..topology.len())
            .map(|i| {
                topology
                    .get(&CorticalSubUnitIndex::from(i as u8))
                    .map(|t| t.channel_dimensions_default[0] as i32)
                    .unwrap_or(1)
            })
            .collect()
    }

    fn boxes(
        topology: &HashMap<CorticalSubUnitIndex, UnitTopology>,
        units: std::ops::Range<u16>,
    ) -> Vec<(String, Box3)> {
        let templates = templates_of(topology);
        let widths = default_dims(topology);
        let mut out = Vec::new();
        for u in units {
            let pos = unit_sub_area_positions(u, &templates, &widths, true);
            for (i, p) in pos.iter().enumerate() {
                let (Some((x, y, z)), Some(t)) =
                    (p, topology.get(&CorticalSubUnitIndex::from(i as u8)))
                else {
                    continue;
                };
                let d = t.channel_dimensions_default;
                out.push((
                    format!("unit{u}/sub{i}"),
                    (
                        (*x, *y, *z),
                        (x + d[0] as i32, y + d[1] as i32, z + d[2] as i32),
                    ),
                ));
            }
        }
        out
    }

    fn overlap(a: &Box3, b: &Box3) -> bool {
        a.0 .0 < b.1 .0
            && b.0 .0 < a.1 .0
            && a.0 .1 < b.1 .1
            && b.0 .1 < a.1 .1
            && a.0 .2 < b.1 .2
            && b.0 .2 < a.1 .2
    }

    /// Every template area of every sensor and motor type (unit 0 only).
    fn all_template_boxes(
        skip_sensor: SensoryCorticalUnit,
        skip_motor: MotorCorticalUnit,
    ) -> Vec<(String, Box3)> {
        let mut out = Vec::new();
        for s in SensoryCorticalUnit::list_all() {
            if *s == skip_sensor {
                continue;
            }
            for (idx, t) in s.get_unit_default_topology() {
                let p = t.relative_position;
                let d = t.channel_dimensions_default;
                out.push((
                    format!("{:?}/{}", s, idx.get()),
                    (
                        (p[0], p[1], p[2]),
                        (p[0] + d[0] as i32, p[1] + d[1] as i32, p[2] + d[2] as i32),
                    ),
                ));
            }
        }
        for m in MotorCorticalUnit::list_all() {
            if *m == skip_motor {
                continue;
            }
            for (idx, t) in m.get_unit_default_topology() {
                let p = t.relative_position;
                let d = t.channel_dimensions_default;
                out.push((
                    format!("{:?}/{}", m, idx.get()),
                    (
                        (p[0], p[1], p[2]),
                        (p[0] + d[0] as i32, p[1] + d[1] as i32, p[2] + d[2] as i32),
                    ),
                ));
            }
        }
        out
    }

    fn assert_clean(own: &[(String, Box3)], others: &[(String, Box3)]) {
        for (i, (na, a)) in own.iter().enumerate() {
            for (nb, b) in own.iter().skip(i + 1) {
                assert!(!overlap(a, b), "{na} overlaps {nb}: {a:?} {b:?}");
            }
            for (nb, b) in others {
                assert!(!overlap(a, b), "{na} overlaps template {nb}: {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn midi_input_units_stack_in_rows_with_expression_beside_notes() {
        let topo = SensoryCorticalUnit::MidiInput.get_unit_default_topology();
        let templates = templates_of(&topo);
        let widths = default_dims(&topo);
        let mut previous_y = None;
        for u in 0..3u16 {
            let pos = unit_sub_area_positions(u, &templates, &widths, true);
            let notes = pos[0].unwrap();
            let expr = pos[1].unwrap();
            assert_eq!(notes.1, expr.1, "unit {u} rows differ");
            assert_eq!(notes.2, expr.2);
            assert_eq!(
                expr.0,
                notes.0 - 5 - AREA_GAP_VOXELS,
                "expression beside notes"
            );
            if let Some(py) = previous_y {
                assert_eq!(notes.1 - py, 20, "stride");
            }
            previous_y = Some(notes.1);
        }
    }

    #[test]
    fn midi_output_units_stack_in_rows_with_expression_beside_notes() {
        let topo = MotorCorticalUnit::MidiOutput.get_unit_default_topology();
        let templates = templates_of(&topo);
        let widths = default_dims(&topo);
        let mut previous_y = None;
        for u in 0..3u16 {
            let pos = unit_sub_area_positions(u, &templates, &widths, true);
            let notes = pos[0].unwrap();
            let expr = pos[1].unwrap();
            assert_eq!(notes.1, expr.1);
            assert_eq!(expr.0, notes.0 - 5 - AREA_GAP_VOXELS);
            if let Some(py) = previous_y {
                assert_eq!(notes.1 - py, 20);
            }
            previous_y = Some(notes.1);
        }
    }

    #[test]
    fn midi_input_units_0_to_3_do_not_overlap_each_other_or_any_template() {
        let topo = SensoryCorticalUnit::MidiInput.get_unit_default_topology();
        let own = boxes(&topo, 0..4);
        let others = all_template_boxes(
            SensoryCorticalUnit::MidiInput,
            MotorCorticalUnit::MidiOutput,
        );
        assert_clean(&own, &others);
    }

    #[test]
    fn midi_output_units_0_to_3_do_not_overlap_each_other_or_any_template() {
        let topo = MotorCorticalUnit::MidiOutput.get_unit_default_topology();
        let own = boxes(&topo, 0..4);
        let others = all_template_boxes(
            SensoryCorticalUnit::MidiInput,
            MotorCorticalUnit::MidiOutput,
        );
        assert_clean(&own, &others);
    }

    #[test]
    fn segmented_vision_layout_is_unchanged() {
        let topo = SensoryCorticalUnit::SegmentedVision.get_unit_default_topology();
        let templates = templates_of(&topo);
        let widths = default_dims(&topo);
        for u in [0u16, 1, 2] {
            let pos = unit_sub_area_positions(u, &templates, &widths, false);
            for (i, p) in pos.iter().enumerate() {
                let t = topo
                    .get(&CorticalSubUnitIndex::from(i as u8))
                    .unwrap()
                    .relative_position;
                assert_eq!(p.unwrap(), (t[0], t[1], t[2]), "unit {u} sub {i}");
            }
        }
    }

    #[test]
    fn raw_imu_order_is_unchanged() {
        let topo = SensoryCorticalUnit::RawIMU.get_unit_default_topology();
        let templates = templates_of(&topo);
        let widths = default_dims(&topo);
        let pos = unit_sub_area_positions(0, &templates, &widths, true);
        let xs: Vec<i32> = pos.iter().map(|p| p.unwrap().0).collect();
        assert!(xs.windows(2).all(|w| w[1] > w[0]), "ascending x: {xs:?}");
        assert_eq!(xs[0], 70);
        // Sub-areas do not overlap (each is 6 wide).
        assert!(xs.windows(2).all(|w| w[1] >= w[0] + 6 + AREA_GAP_VOXELS));
    }

    #[test]
    fn tall_unit_stride_clears_its_height() {
        let templates = vec![Some(SubAreaTemplate {
            position: [0, 60, -20],
            height: 128,
        })];
        let stride = unit_stride_y(&templates);
        assert_eq!(stride, 128 + AREA_GAP_VOXELS);
        let y0 = unit_sub_area_positions(0, &templates, &[128], true)[0]
            .unwrap()
            .1;
        let y1 = unit_sub_area_positions(1, &templates, &[128], true)[0]
            .unwrap()
            .1;
        assert!(y1 >= y0 + 128 + AREA_GAP_VOXELS);
        // Real 128-high motor template.
        let topo = MotorCorticalUnit::list_all()
            .iter()
            .map(|m| m.get_unit_default_topology())
            .find(|t| t.values().any(|u| u.channel_dimensions_default[1] == 128))
            .expect("a 128-high motor template exists");
        assert!(unit_stride_y(&templates_of(&topo)) >= 138);
    }

    #[test]
    fn small_units_keep_the_minimum_stride() {
        let templates = vec![Some(SubAreaTemplate {
            position: [0, 0, 0],
            height: 1,
        })];
        assert_eq!(unit_stride_y(&templates), MIN_UNIT_STRIDE_Y);
    }
}
