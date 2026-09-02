use feagi_data::data_channels::data_cycler::DataCycleEndpoint;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub const DEFAULT_MAX_NUMBER_OF_AGENTS_PER_DATA: usize  = 8;
// TODO on embedded builds turn the above to 1!
// TODO inline error checking

/// Defines the types of data exchanged from the NPU (sensor, motor, visualization).
pub trait DataInterfaceChannelSet<FIQ: FeagiIndexQuantization>: Default {
    /// Sends Visualization Data
    type Visualizer: DataCycleEndpoint<()>;
    /// Sends Motor Data
    type Motor: DataCycleEndpoint<()>;
    /// Receives Sensor Data
    type Sensor: DataCycleEndpoint<()>;

    /// Access the Visualizer Channels
    fn visualizer_channels(&mut self) -> &mut [Self::Visualizer];

    /// Access the Motor Channels
    fn motor_channels(&mut self) -> &mut [Self::Motor];

    /// Access the Sensor Channels
    fn sensor_channels(&mut self) -> &mut [Self::Sensor];

    // TODO insert agent, remove agent
}

