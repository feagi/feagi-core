//! Tests for neuron voxel structures

use feagi_data_structures::neuron_voxels::xyzp::{
    NeuronVoxelXYZP, NeuronVoxelXYZPArrays, CorticalMappedXYZPNeuronVoxels
};
use feagi_data_structures::genomic::cortical_area::CorticalID;
use std::ops::RangeInclusive;

#[cfg(test)]
mod xyzp_tests {
    use feagi_data_structures::genomic::{SensorCorticalType};
    use super::*;
    //region NeuronVoxelXYZP Tests

    #[test]
    fn test_neuron_voxel_creation() {
        let voxel = NeuronVoxelXYZP::new(10, 20, 30, 0.75);

        assert_eq!(voxel.neuron_voxel_coordinate.x, 10);
        assert_eq!(voxel.neuron_voxel_coordinate.y, 20);
        assert_eq!(voxel.neuron_voxel_coordinate.z, 30);
        assert_eq!(voxel.potential, 0.75);
    }

    #[test]
    fn test_neuron_voxel_as_tuple() {
        let voxel = NeuronVoxelXYZP::new(5, 15, 25, 0.5);
        let (x, y, z, p) = voxel.as_tuple();

        assert_eq!(x, 5);
        assert_eq!(y, 15);
        assert_eq!(z, 25);
        assert_eq!(p, 0.5);
    }

    #[test]
    fn test_neuron_voxel_display() {
        let voxel = NeuronVoxelXYZP::new(1, 2, 3, 0.42);
        let display_str = format!("{}", voxel);

        assert!(display_str.contains("NeuronVoxelXYZP"));
        assert!(display_str.contains("1"));
        assert!(display_str.contains("2"));
        assert!(display_str.contains("3"));
    }

    //endregion

    //region NeuronVoxelXYZPArrays Tests

    #[test]
    fn test_arrays_creation_and_basic_ops() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        assert_eq!(arrays.len(), 0);
        assert!(arrays.is_empty());

        let voxel = NeuronVoxelXYZP::new(1, 2, 3, 0.5);
        arrays.push(&voxel);

        assert_eq!(arrays.len(), 1);
        assert!(!arrays.is_empty());
    }

    #[test]
    fn test_arrays_with_capacity() {
        let arrays = NeuronVoxelXYZPArrays::with_capacity(100);

        assert_eq!(arrays.len(), 0);
        assert_eq!(arrays.capacity(), 100);
        assert_eq!(arrays.spare_capacity(), 100);
    }

    #[test]
    fn test_arrays_push_and_get() {
        let mut arrays = NeuronVoxelXYZPArrays::with_capacity(3);

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.1));
        arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.2));
        arrays.push(&NeuronVoxelXYZP::new(7, 8, 9, 0.3));

        assert_eq!(arrays.len(), 3);

        let voxel = arrays.get(1).unwrap();
        assert_eq!(voxel.neuron_voxel_coordinate.x, 4);
        assert_eq!(voxel.neuron_voxel_coordinate.y, 5);
        assert_eq!(voxel.neuron_voxel_coordinate.z, 6);
        assert_eq!(voxel.potential, 0.2);
    }

    #[test]
    fn test_arrays_push_raw() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push_raw(10, 20, 30, 0.7);

        assert_eq!(arrays.len(), 1);
        let voxel = arrays.get(0).unwrap();
        assert_eq!(voxel.neuron_voxel_coordinate.x, 10);
        assert_eq!(voxel.potential, 0.7);
    }

    #[test]
    fn test_arrays_get_components() {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push_raw(5, 10, 15, 0.9);

        assert_eq!(arrays.get_x(0).unwrap(), 5);
        assert_eq!(arrays.get_y(0).unwrap(), 10);
        assert_eq!(arrays.get_z(0).unwrap(), 15);
        assert_eq!(arrays.get_p(0).unwrap(), 0.9);
    }

    #[test]
    fn test_arrays_pop() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
        arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));

        let popped = arrays.pop().unwrap();
        assert_eq!(popped.neuron_voxel_coordinate.x, 4);
        assert_eq!(arrays.len(), 1);

        let popped = arrays.pop().unwrap();
        assert_eq!(popped.neuron_voxel_coordinate.x, 1);
        assert_eq!(arrays.len(), 0);

        assert!(arrays.pop().is_none());
    }

    #[test]
    fn test_arrays_clear() {
        let mut arrays = NeuronVoxelXYZPArrays::with_capacity(10);

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
        arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));

        assert_eq!(arrays.len(), 2);

        arrays.clear();

        assert_eq!(arrays.len(), 0);
        assert!(arrays.is_empty());
        assert_eq!(arrays.capacity(), 10); // Capacity preserved
    }

    #[test]
    fn test_arrays_reserve() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.reserve(50);
        assert!(arrays.capacity() >= 50);
    }

    #[test]
    fn test_arrays_ensure_capacity() {
        let mut arrays = NeuronVoxelXYZPArrays::with_capacity(10);

        arrays.ensure_capacity(5);  // Already have 10, should do nothing
        assert_eq!(arrays.capacity(), 10);

        arrays.ensure_capacity(20); // Need to expand
        assert!(arrays.capacity() >= 20);
    }

    #[test]
    fn test_arrays_shrink_to_fit() {
        let mut arrays = NeuronVoxelXYZPArrays::with_capacity(100);

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
        arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));

        arrays.shrink_to_fit();

        assert_eq!(arrays.capacity(), 2);
        assert_eq!(arrays.len(), 2);
    }

    #[test]
    fn test_arrays_iter() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.1));
        arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.2));
        arrays.push(&NeuronVoxelXYZP::new(7, 8, 9, 0.3));

        let collected: Vec<_> = arrays.iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[1].neuron_voxel_coordinate.x, 4);
    }

    #[test]
    fn test_arrays_enumerate() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push(&NeuronVoxelXYZP::new(10, 20, 30, 0.5));
        arrays.push(&NeuronVoxelXYZP::new(40, 50, 60, 0.7));

        let indexed: Vec<_> = arrays.enumerate().collect();

        assert_eq!(indexed.len(), 2);
        assert_eq!(indexed[0].0, 0);
        assert_eq!(indexed[0].1.neuron_voxel_coordinate.x, 10);
        assert_eq!(indexed[1].0, 1);
        assert_eq!(indexed[1].1.neuron_voxel_coordinate.x, 40);
    }

    #[test]
    fn test_arrays_into_iter() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
        arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));

        let collected: Vec<_> = arrays.into_iter().collect();

        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].neuron_voxel_coordinate.x, 1);
        assert_eq!(collected[1].potential, 0.7);
    }

    #[test]
    fn test_arrays_from_vectors() {
        let x = vec![1, 2, 3];
        let y = vec![4, 5, 6];
        let z = vec![7, 8, 9];
        let p = vec![0.1, 0.2, 0.3];

        let arrays = NeuronVoxelXYZPArrays::new_from_vectors(x, y, z, p).unwrap();

        assert_eq!(arrays.len(), 3);
        let voxel = arrays.get(1).unwrap();
        assert_eq!(voxel.neuron_voxel_coordinate.x, 2);
        assert_eq!(voxel.potential, 0.2);
    }

    #[test]
    fn test_arrays_from_vectors_mismatched_lengths() {
        let x = vec![1, 2];
        let y = vec![4, 5, 6]; // Different length
        let z = vec![7, 8];
        let p = vec![0.1, 0.2];

        let result = NeuronVoxelXYZPArrays::new_from_vectors(x, y, z, p);

        assert!(result.is_err());
    }

    #[test]
    fn test_arrays_from_ndarrays() {
        use ndarray::Array1;

        let x_nd = Array1::from_vec(vec![1, 2, 3]);
        let y_nd = Array1::from_vec(vec![4, 5, 6]);
        let z_nd = Array1::from_vec(vec![7, 8, 9]);
        let p_nd = Array1::from_vec(vec![0.1, 0.2, 0.3]);

        let arrays = NeuronVoxelXYZPArrays::new_from_ndarrays(x_nd, y_nd, z_nd, p_nd).unwrap();

        assert_eq!(arrays.len(), 3);
    }

    #[test]
    fn test_arrays_copy_as_neuron_xyzp_vec() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
        arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));

        let vec = arrays.copy_as_neuron_xyzp_vec();

        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0].neuron_voxel_coordinate.x, 1);
        assert_eq!(vec[1].potential, 0.7);
    }

    #[test]
    fn test_arrays_copy_as_tuple_of_nd_arrays() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
        arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));

        let (x, y, z, p) = arrays.copy_as_tuple_of_nd_arrays();

        assert_eq!(x[0], 1);
        assert_eq!(y[1], 5);
        assert_eq!(z[0], 3);
        assert_eq!(p[1], 0.7);
    }

    #[test]
    fn test_arrays_get_size_in_bytes() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
        arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));

        // Each voxel is 16 bytes (3 u32s + 1 f32)
        assert_eq!(arrays.get_size_in_number_of_bytes(), 32);
    }

    #[test]
    fn test_arrays_borrow_xyzp_vectors() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push_raw(1, 2, 3, 0.5);
        arrays.push_raw(4, 5, 6, 0.7);

        let (x, y, z, p) = arrays.borrow_xyzp_vectors();

        assert_eq!(x.len(), 2);
        assert_eq!(x[0], 1);
        assert_eq!(y[1], 5);
        assert_eq!(z[0], 3);
        assert_eq!(p[1], 0.7);
    }

    #[test]
    fn test_arrays_filter_by_location_bounds() {
        let mut arrays = NeuronVoxelXYZPArrays::new();

        arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.1));
        arrays.push(&NeuronVoxelXYZP::new(5, 6, 7, 0.2));
        arrays.push(&NeuronVoxelXYZP::new(10, 11, 12, 0.3));
        arrays.push(&NeuronVoxelXYZP::new(15, 16, 17, 0.4));

        let filtered = arrays.filter_neurons_by_location_bounds(
            RangeInclusive::new(3, 12),
            RangeInclusive::new(5, 13),
            RangeInclusive::new(6, 14)
        ).unwrap();

        assert_eq!(filtered.len(), 2); // Only voxels at (5,6,7) and (10,11,12) match

        let first = filtered.get(0).unwrap();
        assert_eq!(first.neuron_voxel_coordinate.x, 5);

        let second = filtered.get(1).unwrap();
        assert_eq!(second.neuron_voxel_coordinate.x, 10);
    }

    #[test]
    fn test_arrays_update_vectors_from_external() {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push_raw(1, 2, 3, 0.5);

        let result = arrays.update_vectors_from_external(|x, y, z, p| {
            x[0] = 10;
            y[0] = 20;
            z[0] = 30;
            p[0] = 0.9;
            Ok(())
        });

        assert!(result.is_ok());

        let voxel = arrays.get(0).unwrap();
        assert_eq!(voxel.neuron_voxel_coordinate.x, 10);
        assert_eq!(voxel.neuron_voxel_coordinate.y, 20);
        assert_eq!(voxel.neuron_voxel_coordinate.z, 30);
        assert_eq!(voxel.potential, 0.9);
    }

    //endregion

    //region CorticalMappedXYZPNeuronVoxels Tests

    #[test]
    fn test_cortical_mapped_creation() {
        let mapped = CorticalMappedXYZPNeuronVoxels::new();

        assert_eq!(mapped.len(), 0);
        assert!(mapped.is_empty());
    }

    #[test]
    fn test_cortical_mapped_with_capacity() {
        let mapped = CorticalMappedXYZPNeuronVoxels::new_with_capacity(50);

        assert_eq!(mapped.len(), 0);
        assert!(mapped.capacity() >= 50);
    }

    #[test]
    fn test_cortical_mapped_insert_and_get() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let cortical_id = CorticalID::from_string("iic400".into()).unwrap();
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push_raw(1, 2, 3, 0.5);

        mapped.insert(cortical_id, arrays);

        assert_eq!(mapped.len(), 1);
        assert!(mapped.contains_cortical_id(&cortical_id));

        let retrieved = mapped.get_neurons_of(&cortical_id).unwrap();
        assert_eq!(retrieved.len(), 1);
    }

    #[test]
    fn test_cortical_mapped_get_mut() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let cortical_id = SensorCorticalType::ImageCameraCenterAbsolute.to_cortical_id(0.into());
        let arrays = NeuronVoxelXYZPArrays::new();

        mapped.insert(cortical_id, arrays);

        let neurons_mut = mapped.get_neurons_of_mut(&cortical_id).unwrap();
        neurons_mut.push_raw(10, 20, 30, 0.8);

        let neurons = mapped.get_neurons_of(&cortical_id).unwrap();
        assert_eq!(neurons.len(), 1);
    }

    #[test]
    fn test_cortical_mapped_remove() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let cortical_id = SensorCorticalType::ImageCameraCenterAbsolute.to_cortical_id(0.into());
        mapped.insert(cortical_id, NeuronVoxelXYZPArrays::new());

        assert_eq!(mapped.len(), 1);

        let removed = mapped.remove(cortical_id);
        assert!(removed.is_some());
        assert_eq!(mapped.len(), 0);
    }

    #[test]
    fn test_cortical_mapped_clear() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let id1 = SensorCorticalType::ImageCameraCenterAbsolute.to_cortical_id(0.into());
        let id2 = SensorCorticalType::ImageCameraCenterIncremental.to_cortical_id(0.into());

        mapped.insert(id1, NeuronVoxelXYZPArrays::new());
        mapped.insert(id2, NeuronVoxelXYZPArrays::new());

        assert_eq!(mapped.len(), 2);

        mapped.clear();

        assert_eq!(mapped.len(), 0);
        assert!(mapped.is_empty());
    }

    #[test]
    fn test_cortical_mapped_clear_neurons_only() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let cortical_id = SensorCorticalType::ImageCameraCenterAbsolute.to_cortical_id(0.into());
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push_raw(1, 2, 3, 0.5);

        mapped.insert(cortical_id, arrays);

        mapped.clear_neurons_only();

        assert_eq!(mapped.len(), 1); // Still has the cortical area
        let neurons = mapped.get_neurons_of(&cortical_id).unwrap();
        assert_eq!(neurons.len(), 0); // But neurons are cleared
    }

    #[test]
    fn test_cortical_mapped_iter() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let id1 = SensorCorticalType::ImageCameraCenterAbsolute.to_cortical_id(0.into());
        let id2 = SensorCorticalType::ImageCameraCenterIncremental.to_cortical_id(0.into());

        let mut arrays1 = NeuronVoxelXYZPArrays::new();
        arrays1.push_raw(1, 2, 3, 0.5);

        let mut arrays2 = NeuronVoxelXYZPArrays::new();
        arrays2.push_raw(4, 5, 6, 0.7);

        mapped.insert(id1, arrays1);
        mapped.insert(id2, arrays2);

        let count = mapped.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_cortical_mapped_iter_mut() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let cortical_id = CorticalID::from_string("mut123".into()).unwrap();
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push_raw(1, 2, 3, 0.5);

        mapped.insert(cortical_id, arrays);

        for neurons in mapped.iter_mut() {
            neurons.push_raw(10, 20, 30, 0.9);
        }

        let neurons = mapped.get_neurons_of(&cortical_id).unwrap();
        assert_eq!(neurons.len(), 2);
    }

    #[test]
    fn test_cortical_mapped_keys() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let id1 = SensorCorticalType::ImageCameraCenterAbsolute.to_cortical_id(0.into());
        let id2 = SensorCorticalType::ImageCameraCenterIncremental.to_cortical_id(0.into());

        mapped.insert(id1, NeuronVoxelXYZPArrays::new());
        mapped.insert(id2, NeuronVoxelXYZPArrays::new());

        let keys: Vec<_> = mapped.keys().collect();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_cortical_mapped_into_iter() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let id1 = SensorCorticalType::ImageCameraCenterAbsolute.to_cortical_id(0.into());
        let id2 = SensorCorticalType::ImageCameraCenterIncremental.to_cortical_id(0.into());

        mapped.insert(id1, NeuronVoxelXYZPArrays::new());
        mapped.insert(id2, NeuronVoxelXYZPArrays::new());

        let collected: Vec<_> = mapped.into_iter().collect();
        assert_eq!(collected.len(), 2);
    }

    #[test]
    fn test_cortical_mapped_ensure_clear_and_borrow_mut() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        let cortical_id = SensorCorticalType::ImageCameraCenterAbsolute.to_cortical_id(0.into());

        // First call - creates new empty array
        let neurons = mapped.ensure_clear_and_borrow_mut(&cortical_id);
        neurons.push_raw(1, 2, 3, 0.5);
        assert_eq!(neurons.len(), 1);

        // Second call - clears existing array
        let neurons = mapped.ensure_clear_and_borrow_mut(&cortical_id);
        assert_eq!(neurons.len(), 0); // Should be cleared

        neurons.push_raw(10, 20, 30, 0.9);
        assert_eq!(neurons.len(), 1);
    }

    #[test]
    fn test_cortical_mapped_reserve_and_shrink() {
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new();

        mapped.reserve(50);
        assert!(mapped.capacity() >= 50);

        let id1 = SensorCorticalType::ImageCameraCenterAbsolute.to_cortical_id(0.into());
        mapped.insert(id1, NeuronVoxelXYZPArrays::new());

        mapped.shrink_to_fit();
        assert!(mapped.capacity() >= 1);
    }

    //endregion
}

