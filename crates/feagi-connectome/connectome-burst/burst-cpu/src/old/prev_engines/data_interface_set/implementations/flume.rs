use feagi_basis::prelude::*;
use feagi_basis::thread_messaging::data_cycler::implementations::flume::FlumeDataCycleEndpoint;
use crate::data_interface_set::data_interface_set::DataInterfaceChannelSet;

/// Channels using mpmc channels from the `Flume` crate
pub struct FlumeDataInterfaceSet<FIQ: FeagiIndexQuantization, const CAP: usize>
{
    visualizers: heapless::Vec<FlumeDataCycleEndpoint<()>, CAP>,
    motors: heapless::Vec<FlumeDataCycleEndpoint<()>, CAP>,
    sensors: heapless::Vec<FlumeDataCycleEndpoint<()>, CAP>,
    _p: core::marker::PhantomData<FIQ>
}

impl<FIQ: FeagiIndexQuantization, const CAP: usize> DataInterfaceChannelSet<FIQ> for FlumeDataInterfaceSet<FIQ, CAP> {
    type Visualizer = FlumeDataCycleEndpoint<()>;
    type Motor = FlumeDataCycleEndpoint<()>;
    type Sensor = FlumeDataCycleEndpoint<()>;

    fn visualizer_channels(&mut self) -> &mut [Self::Visualizer] {
        self.visualizers.as_mut_slice()
    }

    fn motor_channels(&mut self) -> &mut [Self::Motor] {
        self.motors.as_mut_slice()
    }

    fn sensor_channels(&mut self) -> &mut [Self::Sensor] {
        self.sensors.as_mut_slice()
    }
}

impl<FIQ: FeagiIndexQuantization, const CAP: usize> Default for FlumeDataInterfaceSet<FIQ, CAP> {
    fn default() -> Self {
        Self {
            visualizers: heapless::Vec::new(),
            motors: heapless::Vec::new(),
            sensors: heapless::Vec::new(),
            _p: core::marker::PhantomData
        }
    }
}