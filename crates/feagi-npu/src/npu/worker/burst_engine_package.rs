use feagi_data::data_channels::data_cycler::DataCycleEndpoint;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_engines::BurstEngineEnum;

pub enum BurstEngineDataInterface<
    FIQ: FeagiIndexQuantization,
    VisualizationTransmitter: DataCycleEndpoint<u8>,
    MotorTransmitter: DataCycleEndpoint<u8>,
    SensorReceiver: DataCycleEndpoint<u8>,
>
{
    SensorReceiver(SensorReceiver),
    MotorTransmitter(MotorTransmitter),
    VisualizationTransmitter(VisualizationTransmitter),
}

pub struct BurstEnginePackage<
    FIQ: FeagiIndexQuantization,
    VisualizationTransmitter: DataCycleEndpoint<u8>,
    MotorTransmitter: DataCycleEndpoint<u8>,
    SensorReceiver: DataCycleEndpoint<u8>,
>
{
    pub burst_engine: BurstEngineEnum<FIQ>,
    pub interfaces: Vec<BurstEngineDataInterface<FIQ, VisualizationTransmitter, MotorTransmitter, SensorReceiver>>,
}