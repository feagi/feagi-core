

#[cfg(test)]
mod burst_engine_rayon_tests
{
    // TODO Actual Injection!

    use feagi_npu_structures::neural_processing_unit_data_structures::dynamic_burst_engine_interface::npu_requests::npu_request::NPURequest;
    use feagi_structures::neuron_voxels::bit_32::NeuronVoxelDimensions;

    #[test]
    fn rayon_full_test_suite()
    {


        let area_alpha = NPURequest::cortical_area_create_custom(
            NeuronVoxelDimensions::new_unchecked(32, 32, 32)
            0,
            (),
            Default::default())

        }
    }












}