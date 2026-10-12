
// NOTE: Do not make this a u8! Instead, we may make this representable as some other uint manually
// down the line


pub enum FeagiCapabilities {
    /// Processes neuron data in bursts
    Brain,
    /// Can produce realtime neuron firing visualization data
    VisualizationProducer,
    /// Can visualize neuron firing data for developers
    VisualizationConsumer,
    /// Can take in SenoriMotor Data
    SensoriMotorSink

}



// TODO for some of these, nest a uint bitflag for additional capabilities

// TODO actually lets just move the whole thing to a bitflag set struct with a nested enum
// representation