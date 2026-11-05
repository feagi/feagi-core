#![cfg(feature = "std")]

use feagi_basis::neurons::DimensionalCorticalAreaDimensions;
use feagi_basis::prelude::SpatialCoordinate;
use feagi_basis::voxels::{CorticalAreaVoxelCoordinateAxisPosition, CorticalAreaVoxelCoordinates};
use neural_basis::connectivity_rule::voxel_to_voxel::vector::{
    ConnectivityGeneratorError, ConnectivityRuleError, ConnectivityRuleVector, ConnectivityRuleVectorElement,
};
use neural_basis::connectivity_rule::voxel_to_voxel::{ConnectivityGeneratorVoxelToVoxel, ConnectivityRuleVoxelToVoxel};
use neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use neural_basis::neuron_model::neuron_layout::NeuronLayout;
use neural_basis::quantization_levels::feagi_index_quantization::{
    FeagiIndexQuantization, FeagiIndexQuantizationLow, FeagiIndexQuantizationStandard,
};

// Low quantization, so that synapse counts can overflow within a reasonable test
type Fiq = FeagiIndexQuantizationLow;
type Rule = ConnectivityRuleVector<Fiq>;
type Element = ConnectivityRuleVectorElement<Fiq>;
type Dims = DimensionalCorticalAreaDimensions<u16>;
type Voxel = [u16; 3];

fn dims(data: [u16; 4]) -> Dims {
    Dims::new_dimensions(data).unwrap()
}

fn coordinates(voxel: Voxel) -> CorticalAreaVoxelCoordinates<u16> {
    SpatialCoordinate::new_coordinate(voxel.map(CorticalAreaVoxelCoordinateAxisPosition::new))
}

fn element(source: Voxel, destination: Voxel) -> Element {
    Element { source: coordinates(source), destination: coordinates(destination) }
}

fn rule(pairs: &[(Voxel, Voxel)]) -> Rule {
    Rule::new(pairs.iter().map(|(source, destination)| element(*source, *destination)).collect()).unwrap()
}

/// Generates all pairs as plain numbers, checking that the reported counts agree with the output
fn generate(rule: &Rule, source_dims: Dims, destination_dims: Dims) -> Vec<(u16, u16)> {
    let generator = rule.spawn_generator(source_dims, destination_dims).unwrap();
    let iter = generator.iter();
    let reported = iter.len();
    let pairs: Vec<(u16, u16)> = iter.map(|(source, destination)| (source.deref(), destination.deref())).collect();
    assert_eq!(pairs.len(), reported);
    assert_eq!(pairs.len(), generator.number_of_neuron_pairs().deref() as usize);
    pairs
}

fn skipped(rule: &Rule, source_dims: Dims, destination_dims: Dims) -> usize {
    rule.spawn_generator(source_dims, destination_dims).unwrap().number_of_skipped_elements()
}

//region Rule construction

#[test]
fn rule_rejects_adjacent_repeat() {
    let result = Rule::new(vec![element([0, 0, 0], [1, 0, 0]), element([0, 0, 0], [1, 0, 0])]);
    assert!(matches!(result, Err(ConnectivityRuleError::DuplicateMappings)));
}

#[test]
fn rule_rejects_non_adjacent_repeat() {
    let result = Rule::new(vec![
        element([0, 0, 0], [1, 0, 0]),
        element([1, 0, 0], [0, 1, 0]),
        element([0, 0, 0], [1, 0, 0]),
    ]);
    assert!(matches!(result, Err(ConnectivityRuleError::DuplicateMappings)));
}

#[test]
fn rule_allows_shared_voxels_and_reversed_pairs() {
    assert!(Rule::new(vec![
        element([0, 0, 0], [1, 0, 0]),
        element([0, 0, 0], [0, 1, 0]),
        element([1, 0, 0], [0, 0, 0]),
    ]).is_ok());
}

#[test]
fn rule_allows_no_elements() {
    assert!(Rule::new(vec![]).is_ok());
}

//endregion

//region Pair generation

#[test]
fn single_neuron_voxels_make_one_pair() {
    let rule = rule(&[([1, 0, 0], [0, 1, 0])]);
    // Source stride of X is 1 with a YZ of 1, destination stride of Y is 1 with a Z of 1
    assert_eq!(generate(&rule, dims([2, 1, 1, 1]), dims([1, 2, 1, 1])), vec![(1, 1)]);
}

#[test]
fn densities_of_two_and_three_make_six_pairs() {
    let rule = rule(&[([0, 0, 0], [0, 0, 0])]);
    assert_eq!(
        generate(&rule, dims([1, 1, 1, 2]), dims([1, 1, 1, 3])),
        vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2)],
    );
}

#[test]
fn every_element_is_expanded_in_rule_order() {
    let rule = rule(&[([1, 1, 0], [2, 0, 0]), ([0, 0, 0], [0, 0, 0])]);
    assert_eq!(
        generate(&rule, dims([2, 2, 1, 2]), dims([3, 1, 1, 3])),
        vec![
            (6, 6), (6, 7), (6, 8), (7, 6), (7, 7), (7, 8),
            (0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2),
        ],
    );
}

#[test]
fn indexes_lead_back_to_the_voxels_of_the_element() {
    let (source_dims, destination_dims) = (dims([3, 2, 4, 2]), dims([2, 5, 3, 3]));
    let (source_voxel, destination_voxel) = ([2, 1, 3], [1, 4, 2]);
    let rule = rule(&[(source_voxel, destination_voxel)]);
    let source_layout = NeuronLayoutVoxel::<Fiq>::new(source_dims);
    let destination_layout = NeuronLayoutVoxel::<Fiq>::new(destination_dims);

    let generator = rule.spawn_generator(source_dims, destination_dims).unwrap();
    let mut expected_densities = Vec::new();
    for s in 0..2u16 {
        for d in 0..3u16 {
            expected_densities.push((s, d));
        }
    }

    let actual: Vec<([u16; 4], [u16; 4])> = generator.iter()
        .map(|(source, destination)| (
            *source_layout.get_neuron_layout_context(&source).as_slice(),
            *destination_layout.get_neuron_layout_context(&destination).as_slice(),
        ))
        .collect();
    let expected: Vec<([u16; 4], [u16; 4])> = expected_densities.iter()
        .map(|(s, d)| ([2, 1, 3, *s], [1, 4, 2, *d]))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn iterating_again_gives_the_same_pairs() {
    let rule = rule(&[([0, 0, 0], [0, 0, 0]), ([1, 0, 0], [0, 0, 0])]);
    let generator = rule.spawn_generator(dims([2, 1, 1, 2]), dims([1, 1, 1, 2])).unwrap();
    let first: Vec<_> = generator.iter().collect();
    let second: Vec<_> = generator.iter().collect();
    assert_eq!(first, second);
}

#[test]
fn one_rule_can_be_used_with_different_dimensions() {
    let rule = rule(&[([0, 0, 0], [0, 0, 0])]);
    assert_eq!(generate(&rule, dims([1, 1, 1, 1]), dims([1, 1, 1, 1])).len(), 1);
    assert_eq!(generate(&rule, dims([1, 1, 1, 2]), dims([1, 1, 1, 2])).len(), 4);
}

#[test]
fn generic_code_can_reach_the_generator() {
    fn pair_count<F: FeagiIndexQuantization, R: ConnectivityRuleVoxelToVoxel<F>>(
        rule: &R,
        source_dims: DimensionalCorticalAreaDimensions<F::NeuronIndexQuant>,
        destination_dims: DimensionalCorticalAreaDimensions<F::NeuronIndexQuant>,
    ) -> usize {
        let generator = rule.spawn_generator(source_dims, destination_dims).unwrap();
        let count = generator.iter().len();
        count
    }

    let voxel = SpatialCoordinate::new_coordinate([CorticalAreaVoxelCoordinateAxisPosition::new(0u32); 3]);
    let rule = ConnectivityRuleVector::<FeagiIndexQuantizationStandard>::new(vec![
        ConnectivityRuleVectorElement { source: voxel, destination: voxel },
    ]).unwrap();
    let area = DimensionalCorticalAreaDimensions::<u32>::new_dimensions([1, 1, 1, 2]).unwrap();
    assert_eq!(pair_count(&rule, area, area), 4);
}

//endregion

//region Exact size

#[test]
fn length_counts_down_while_iterating() {
    let rule = rule(&[([0, 0, 0], [0, 0, 0])]);
    let generator = rule.spawn_generator(dims([1, 1, 1, 2]), dims([1, 1, 1, 2])).unwrap();
    let mut iter = generator.iter();
    for remaining in (0..4).rev() {
        assert!(iter.next().is_some());
        assert_eq!(iter.len(), remaining);
    }
    assert!(iter.next().is_none());
    assert_eq!(iter.len(), 0);
}

#[test]
fn length_ignores_skipped_elements() {
    let rule = rule(&[([0, 0, 0], [0, 0, 0]), ([9, 0, 0], [0, 0, 0]), ([0, 0, 0], [0, 1, 0])]);
    let generator = rule.spawn_generator(dims([1, 1, 1, 2]), dims([1, 2, 1, 3])).unwrap();
    assert_eq!(generator.iter().len(), 12);
}

//endregion

//region Skipping

#[test]
fn out_of_range_source_is_skipped() {
    let rule = rule(&[([2, 0, 0], [0, 0, 0]), ([1, 0, 0], [0, 0, 0])]);
    let (source_dims, destination_dims) = (dims([2, 1, 1, 1]), dims([1, 1, 1, 1]));
    assert_eq!(generate(&rule, source_dims, destination_dims), vec![(1, 0)]);
    assert_eq!(skipped(&rule, source_dims, destination_dims), 1);
}

#[test]
fn out_of_range_destination_is_skipped() {
    let rule = rule(&[([0, 0, 0], [0, 0, 2]), ([0, 0, 0], [0, 0, 1])]);
    let (source_dims, destination_dims) = (dims([1, 1, 1, 1]), dims([1, 1, 2, 1]));
    assert_eq!(generate(&rule, source_dims, destination_dims), vec![(0, 1)]);
    assert_eq!(skipped(&rule, source_dims, destination_dims), 1);
}

#[test]
fn nothing_fitting_gives_an_empty_generator() {
    let rule = rule(&[([5, 0, 0], [0, 0, 0]), ([0, 0, 0], [5, 0, 0])]);
    let (source_dims, destination_dims) = (dims([1, 1, 1, 2]), dims([1, 1, 1, 2]));
    assert_eq!(generate(&rule, source_dims, destination_dims), vec![]);
    assert_eq!(skipped(&rule, source_dims, destination_dims), 2);
}

#[test]
fn no_elements_gives_an_empty_generator() {
    let rule = rule(&[]);
    assert_eq!(generate(&rule, dims([1, 1, 1, 1]), dims([1, 1, 1, 1])), vec![]);
    assert_eq!(skipped(&rule, dims([1, 1, 1, 1]), dims([1, 1, 1, 1])), 0);
}

//endregion

//region Overflow

#[test]
fn largest_count_that_fits_the_quantization_is_allowed() {
    let rule = rule(&[([0, 0, 0], [0, 0, 0])]);
    // 255 * 257 = 65535, the max of u16
    let generator = rule.spawn_generator(dims([1, 1, 1, 255]), dims([1, 1, 1, 257])).unwrap();
    assert_eq!(generator.number_of_neuron_pairs().deref(), u16::MAX);
    assert_eq!(generator.iter().len(), u16::MAX as usize);
}

#[test]
fn count_past_the_quantization_is_an_error() {
    let rule = rule(&[([0, 0, 0], [0, 0, 0])]);
    let result = rule.spawn_generator(dims([1, 1, 1, 256]), dims([1, 1, 1, 256]));
    assert!(matches!(result, Err(ConnectivityGeneratorError::TooManySynapses)));
}

#[test]
fn count_overflowing_only_across_elements_is_an_error() {
    let pairs: Vec<(Voxel, Voxel)> = (0..300).map(|x| ([x, 0, 0], [0, 0, 0])).collect();
    let rule = rule(&pairs);
    // 300 elements * 256 pairs = 76800, where a single element's 256 pairs would fit
    let result = rule.spawn_generator(dims([300, 1, 1, 16]), dims([1, 1, 1, 16]));
    assert!(matches!(result, Err(ConnectivityGeneratorError::TooManySynapses)));
}

#[test]
fn skipped_elements_do_not_count_toward_overflow() {
    let pairs: Vec<(Voxel, Voxel)> = (0..300).map(|x| ([x, 0, 0], [0, 0, 0])).collect();
    let rule = rule(&pairs);
    // Only 2 of the 300 elements fit, giving 512 pairs
    let (source_dims, destination_dims) = (dims([2, 1, 1, 16]), dims([1, 1, 1, 16]));
    assert_eq!(generate(&rule, source_dims, destination_dims).len(), 512);
    assert_eq!(skipped(&rule, source_dims, destination_dims), 298);
}

//endregion
