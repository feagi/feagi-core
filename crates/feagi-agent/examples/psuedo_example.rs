use feagi_agent::clients::EmbodimentAgent;
use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, ImageFrameProperties, ImageXYResolution,
};
use feagi_sensorimotor::data_types::ImageFrame;
use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;

fn main() {
    let registration_endpoint = "zmq://127.0.0.1".to_string();
    let mut blocking_agent: EmbodimentAgent = EmbodimentAgent::new().unwrap(); // NOTE: for now this is blocking only!

    // adding devices
    //blocking_agent.get_embodiment_mut().import_device_registrations_as_config_json("JSON GO HERE").unwrap() // This can be one way of doing it

    // You can also just define them one at a time
    blocking_agent
        .get_embodiment_mut()
        .get_motor_cache()
        .gaze_register(
            0.into(),
            1.into(),
            Default::default(),
            10.into(),
            (16, 16).into(),
            Default::default(),
        )
        .unwrap();
    blocking_agent
        .get_embodiment_mut()
        .get_motor_cache()
        .rotary_motor_register(
            0.into(),
            1.into(),
            Default::default(),
            10.into(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap();
    blocking_agent
        .get_embodiment_mut()
        .get_sensor_cache()
        .vision_register(
            0.into(),
            1.into(),
            Default::default(),
            ImageFrameProperties::new(
                ImageXYResolution::new(16, 16).unwrap(),
                ColorSpace::Linear,
                ColorChannelLayout::GrayScale,
            )
            .unwrap(),
        )
        .unwrap();

    // Connect to feagi (Note this is long-term blocking, which can be bad in certain systems)
    blocking_agent
        .connect_to_feagi_zmq(&registration_endpoint)
        .unwrap();

    // As this is blocking, the user needs to manually define a loop. This is a really dirty example of doing it. We will likely want a tokio version that handles this nicer
    loop {
        blocking_agent.poll().unwrap(); // poll network

        // write sensor data
        let mut sensors = blocking_agent.get_embodiment_mut().get_sensor_cache();
        sensors
            .vision_write(
                0.into(),
                0.into(),
                WrappedIOData::ImageFrame(
                    ImageFrame::new(
                        &ColorChannelLayout::GrayScale,
                        &ColorSpace::Linear,
                        &ImageXYResolution::new(16, 16).unwrap(),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();

        // Send sensor data
        blocking_agent.send_encoded_sensor_data().unwrap();

        // NOTE: obviously, we should not be sending sensor data every literal nanosecond. A proper loop setup would time this better, but this is just an example of syntax
    }
}
