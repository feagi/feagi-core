use feagi_basis_genome_proc::on_cortical_template;


on_cortical_template! {
    template {
        Infrared {
            encoded_data_type: Percentage,
            friendly_name: "Infrared Sensor",
            cortical_id_tag: b"abc",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [10, 0, -20],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    io_cortical_generator: Generator
                }
            }
        }
    }
}