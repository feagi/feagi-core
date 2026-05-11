//! Tests for neuron voxel primitives.

use feagi_structures::neuron_collections::::coord_potential::NeuronVoxelXYZP;
use feagi_structures::neuron_collections::::voxel_structs::NeuronVoxelPotential;

#[cfg(test)]
mod xyzp_tests {
    use super::*;

    #[test]
    fn neuron_voxel_creation() {
        let voxel = NeuronVoxelXYZP::new(10u32, 20u32, 30u32, NeuronVoxelPotential(0.75f32));

        assert_eq!(voxel.coordinate.x, 10);
        assert_eq!(voxel.coordinate.y, 20);
        assert_eq!(voxel.coordinate.z, 30);
        assert_eq!(voxel.potential.0, 0.75f32);
    }

    #[test]
    fn neuron_voxel_display_contains_coords() {
        let voxel = NeuronVoxelXYZP::new(1u32, 2u32, 3u32, NeuronVoxelPotential(0.42f32));
        let display_str = format!("{}", voxel);

        assert!(display_str.contains("NeuronVoxelXYZP"));
        assert!(display_str.contains('1'));
        assert!(display_str.contains('2'));
        assert!(display_str.contains('3'));
    }
}

// TODO: NeuronVoxelXYZPSparseVectors / CorticalMappedXYZPNeuronVoxels integration tests (push/get/iter, serde, filters) — types are not exported on this path yet or APIs changed; re-add when collections are public and non-todo.
