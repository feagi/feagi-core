use feagi_basis_genome_proc::make_cortical_template;

// This generates a 'cortical_interface_templates!' exported macro for use in 
// creating interface types

make_cortical_template! {
    template {
        Infrared {
            encoded_data_type: Percentage,
            friendly_name: "Infrared Sensor",
            cortical_id_tag: b"abc",
            comment: "Infrared distance sensor for object detection.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [10, 0, -20],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    comment: "(blank)",
                    io_cortical_generator: Generator,
                }
            }
        },
        Proximity {
            encoded_data_type: Percentage,
            friendly_name: "Proximity Sensor",
            cortical_id_tag: b"pro",
            comment: "Proximity (distance) sensor for object detection.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [20, 0, -20],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        Shock {
            encoded_data_type: Percentage,
            friendly_name: "Shock sensor",
            cortical_id_tag: b"shk",
            comment: "Shock sensor for sensing pain. Useful for training.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [30, 0, -20],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        Battery {
            encoded_data_type: Percentage,
            friendly_name: "Battery Sensor",
            cortical_id_tag: b"bat",
            comment: "Battery level sensor.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [40, 0, -20],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        Servo {
            encoded_data_type: Percentage,
            friendly_name: "Servo Sensor",
            cortical_id_tag: b"svm",
            comment: "Servo position sensor for monitoring actuator position.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [25, 0, -10],
                    channel_dimensions_default: [8, 8, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1024, 1024, 1],
                    io_cortical_generator: Generator,
                }
            }
        },
        AnalogGPIO {
            encoded_data_type: Percentage,
            friendly_name: "Analog GPIO Sensor",
            cortical_id_tag: b"agp",
            comment: "Analog GPIO input such as a reading from Raspberry Pi GPIO pins.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [60, 0, -10],
                    channel_dimensions_default: [8, 8, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1024, 1024, 1],
                    io_cortical_generator: Generator,
                }
            }
        },
        DigitalGPIO {
            encoded_data_type: Boolean,
            friendly_name: "Digital GPIO Sensor",
            cortical_id_tag: b"dgp",
            comment: "Digital GPIO input such as a binary signal from Raspberry Pi GPIO pins.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Boolean,
                    relative_position: [70, 0, -10],
                    channel_dimensions_default: [1, 1, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1],
                    io_cortical_generator: Generator,
                }
            }
        },
        MiscData {
            encoded_data_type: MiscData,
            friendly_name: "Miscellaneous Sensor",
            cortical_id_tag: b"imi",
            comment: "Miscellaneous sensor for signals that do not fit existing cortical units.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Misc,
                    relative_position: [220, 0, -30],
                    channel_dimensions_default: [8, 8, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1024, 1024, 1],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional field omitted (no equivalent here): allowed_frame_change_handling
        TextEnglishInput {
            encoded_data_type: MiscData,
            friendly_name: "Text Input (English)",
            cortical_id_tag: b"ite",
            comment: "Text input encoded as absolute fractional bitplanes along Z (z=0 is MSB).",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Misc,
                    relative_position: [70, 0, -30],
                    channel_dimensions_default: [1, 1, 16],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 32],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional field omitted (no equivalent here): allowed_frame_change_handling
        CountInput {
            encoded_data_type: Percentage,
            friendly_name: "Count Input",
            cortical_id_tag: b"ico",
            comment: "Count input using unsigned percentage encoding (linear, absolute).",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [110, 0, -30],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional fields omitted (no equivalent here): default_firing_threshold, default_mp_charge_accumulation
        Vision {
            encoded_data_type: ImageFrame,
            friendly_name: "Simple Vision",
            cortical_id_tag: b"iim",
            comment: "Camera vision input.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [-100, 30, 0],
                    channel_dimensions_default: [64, 64, 3],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional fields omitted (no equivalent here): default_firing_threshold, default_firing_threshold_increment, default_mp_charge_accumulation
        DepthMap {
            encoded_data_type: MiscData,
            friendly_name: "Depth Map",
            cortical_id_tag: b"dpt",
            comment: "Depth map input where X/Y encode topology and Z encodes quantized depth bins.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Misc,
                    relative_position: [-140, 30, 0],
                    channel_dimensions_default: [64, 64, 64],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional fields omitted (no equivalent here): default_firing_threshold, default_mp_charge_accumulation
        SegmentedVision {
            encoded_data_type: SegmentedImageFrame,
            friendly_name: "Segmented Vision",
            cortical_id_tag: b"svi",
            comment: "Segmented vision processing with high-resolution center and lower-resolution periphery.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [-70, -70, 0],
                    channel_dimensions_default: [32, 32, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    comment: "Lower Left",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [60, -70, 0],
                    channel_dimensions_default: [32, 32, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    comment: "Lower Middle",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [150, -70, 0],
                    channel_dimensions_default: [32, 32, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    comment: "Lower Right",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [-70, 60, 0],
                    channel_dimensions_default: [32, 32, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    comment: "Middle Left",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [0, 0, 0],
                    channel_dimensions_default: [128, 128, 3],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    comment: "Middle Middle",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [150, 60, 0],
                    channel_dimensions_default: [32, 32, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    comment: "Middle Right",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [-70, 150, 0],
                    channel_dimensions_default: [32, 32, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    comment: "Upper Left",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [60, 150, 0],
                    channel_dimensions_default: [32, 32, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    comment: "Upper Middle",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [150, 150, 0],
                    channel_dimensions_default: [32, 32, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    comment: "Upper Right",
                    io_cortical_generator: Generator,
                }
            }
        },
        Accelerometer {
            encoded_data_type: Percentage_3D,
            friendly_name: "Accelerometer",
            cortical_id_tag: b"acc",
            comment: "Accelerometer for relative tracking of position and motion.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: SignedPercentage3D,
                    relative_position: [70, 0, -10],
                    channel_dimensions_default: [3, 1, 10],
                    channel_dimensions_min: [3, 1, 1],
                    channel_dimensions_max: [3, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        Gyroscope {
            encoded_data_type: SignedPercentage_4D,
            friendly_name: "Gyroscope",
            cortical_id_tag: b"gyq",
            comment: "Gyroscope (quaternion) for tracking rotation without gimbal lock.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: SignedPercentage4D,
                    relative_position: [80, 0, -10],
                    channel_dimensions_default: [4, 1, 10],
                    channel_dimensions_min: [4, 1, 1],
                    channel_dimensions_max: [4, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        RawIMU {
            encoded_data_type: RawIMU,
            friendly_name: "Raw IMU",
            cortical_id_tag: b"rim",
            comment: "Raw IMU: composite linear-vector sensor with three sub-cortical-areas (accelerometer + gyroscope + magnetometer), each a 3-axis signed percentage.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: SignedPercentage3D,
                    relative_position: [70, 0, -10],
                    channel_dimensions_default: [3, 1, 10],
                    channel_dimensions_min: [3, 1, 1],
                    channel_dimensions_max: [3, 1, 1024],
                    comment: "Accelerometer",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: SignedPercentage3D,
                    relative_position: [80, 0, -10],
                    channel_dimensions_default: [3, 1, 10],
                    channel_dimensions_min: [3, 1, 1],
                    channel_dimensions_max: [3, 1, 1024],
                    comment: "Gyroscope",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: SignedPercentage3D,
                    relative_position: [90, 0, -10],
                    channel_dimensions_default: [3, 1, 10],
                    channel_dimensions_min: [3, 1, 1],
                    channel_dimensions_max: [3, 1, 1024],
                    comment: "Magnetometer",
                    io_cortical_generator: Generator,
                },
            }
        },
        SmartIMU {
            encoded_data_type: SignedPercentage_4D,
            friendly_name: "Smart IMU",
            cortical_id_tag: b"sim",
            comment: "Smart IMU: orientation as a unit quaternion (w/x/y/z) in one sub-area.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: SignedPercentage4D,
                    relative_position: [100, 0, -10],
                    channel_dimensions_default: [4, 1, 10],
                    channel_dimensions_min: [4, 1, 1],
                    channel_dimensions_max: [4, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional field omitted (no equivalent here): allowed_frame_change_handling
        CartesianPosition {
            encoded_data_type: Percentage_3D,
            friendly_name: "Cartesian Position Sensor",
            cortical_id_tag: b"cpo",
            comment: "Cartesian position sensor: absolute 3D position (x/y/z), each axis normalized.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage3D,
                    relative_position: [115, 0, -10],
                    channel_dimensions_default: [3, 1, 10],
                    channel_dimensions_min: [3, 1, 1],
                    channel_dimensions_max: [3, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        RotaryMotor {
            encoded_data_type: SignedPercentage,
            friendly_name: "Rotary Motor",
            cortical_id_tag: b"mot",
            comment: "Free spinning motor.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: SignedPercentage,
                    relative_position: [-20, 0, -10],
                    channel_dimensions_default: [1, 1, 9],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy area-level frame-change specialization omitted (no equivalent here): Absolute / Incremental split
        PositionalServo {
            encoded_data_type: Percentage,
            friendly_name: "Positional Servo",
            cortical_id_tag: b"pse",
            comment: "Servo position output with absolute and incremental control areas.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [-20, 0, -10],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    comment: "Absolute position channel in legacy implementation.",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [-40, 0, -10],
                    channel_dimensions_default: [2, 1, 10],
                    channel_dimensions_min: [2, 1, 1],
                    channel_dimensions_max: [2, 1, 1024],
                    comment: "Incremental position channel in legacy implementation.",
                    io_cortical_generator: Generator,
                }
            }
        },
        Gaze {
            encoded_data_type: GazeProperties,
            friendly_name: "Gaze Control",
            cortical_id_tag: b"gaz",
            comment: "Gaze control where XY represents center and Z represents relative size.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage2D,
                    relative_position: [10, 0, -10],
                    channel_dimensions_default: [8, 8, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1024, 1024, 1],
                    comment: "Eccentricity",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [0, 0, -10],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    comment: "Modularity",
                    io_cortical_generator: Generator,
                }
            }
        },
        MotorMiscData {
            encoded_data_type: MiscData,
            friendly_name: "Miscellaneous Motor",
            cortical_id_tag: b"mmi",
            comment: "Miscellaneous motor output that does not fit existing cortical units.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Misc,
                    relative_position: [300, 0, -30],
                    channel_dimensions_default: [8, 8, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1024, 1024, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional field omitted (no equivalent here): allowed_frame_change_handling
        TextEnglishOutput {
            encoded_data_type: MiscData,
            friendly_name: "Text Output (English)",
            cortical_id_tag: b"ote",
            comment: "Text output encoded as absolute fractional bitplanes along Z (z=0 is MSB).",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Misc,
                    relative_position: [85, 0, -30],
                    channel_dimensions_default: [1, 1, 16],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 32],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional field omitted (no equivalent here): allowed_frame_change_handling
        CountOutput {
            encoded_data_type: Percentage,
            friendly_name: "Count Output",
            cortical_id_tag: b"oco",
            comment: "Count output using unsigned percentage encoding (linear, absolute).",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [175, 0, -30],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional field omitted (no equivalent here): allowed_frame_change_handling
        ObjectSegmentation {
            encoded_data_type: MiscData,
            friendly_name: "Object Segmentation",
            cortical_id_tag: b"seg",
            comment: "Object semantic segmentation output with bitplane class encoding.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Misc,
                    relative_position: [-200, 0, 0],
                    channel_dimensions_default: [32, 32, 8],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 1024],
                    io_cortical_generator: Generator,
                }
            }
        },
        SimpleVisionOutput {
            encoded_data_type: ImageFrame,
            friendly_name: "Simple Vision",
            cortical_id_tag: b"oim",
            comment: "Visual thoughts output as RGB image generation from brain activity.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: CartesianPlane,
                    relative_position: [-240, 60, -20],
                    channel_dimensions_default: [128, 128, 3],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 3],
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional field omitted: allowed_frame_change_handling
        // Legacy cortical_type_parameter: pose_schema
        PoseEstimation {
            encoded_data_type: PoseEstimationData,
            friendly_name: "Pose Estimation",
            cortical_id_tag: b"pos",
            comment: "Pose estimation output where XY is joint location and Z-depth maps joint ID.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: PoseEstimation,
                    relative_position: [-200, 60, 0],
                    channel_dimensions_default: [64, 64, 17],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 256],
                    io_cortical_generator: Generator,
                }
            }
        },
        DynamicImageProcessing {
            encoded_data_type: ImageFilteringSettings,
            friendly_name: "Image Enhancements",
            cortical_id_tag: b"ifs",
            comment: "Image processing controls for brightness, contrast, and differential filtering.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [0, 0, 0],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    comment: "Brightness",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: Percentage,
                    relative_position: [0, 0, -10],
                    channel_dimensions_default: [1, 1, 10],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [1, 1, 1024],
                    comment: "Contrast",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: Percentage2D,
                    relative_position: [0, 0, -30],
                    channel_dimensions_default: [2, 1, 10],
                    channel_dimensions_min: [2, 1, 1],
                    channel_dimensions_max: [2, 1, 1024],
                    comment: "Per-pixel diff",
                    io_cortical_generator: Generator,
                },
                {
                    io_cortical_data_type: Percentage2D,
                    relative_position: [0, 0, -30],
                    channel_dimensions_default: [2, 1, 10],
                    channel_dimensions_min: [2, 1, 1],
                    channel_dimensions_max: [2, 1, 1024],
                    comment: "Image diff",
                    io_cortical_generator: Generator,
                }
            }
        },
        // Legacy optional field omitted (no equivalent here): allowed_frame_change_handling
        // Legacy IO flag helper omitted (no equivalent here): spatial_pointer_io_flag
        SpatialPointer {
            encoded_data_type: SpatialPointer3D,
            friendly_name: "Spatial Pointer",
            cortical_id_tag: b"ptr",
            comment: "Spatial pointer output decodes activity into a normalized XYZ percentage tuple.",
            io_cortical_areas: {
                {
                    io_cortical_data_type: Percentage3D,
                    relative_position: [210, 0, -30],
                    channel_dimensions_default: [64, 64, 1],
                    channel_dimensions_min: [1, 1, 1],
                    channel_dimensions_max: [4096, 4096, 4096],
                    io_cortical_generator: Generator,
                }
            }
        },
    }
}