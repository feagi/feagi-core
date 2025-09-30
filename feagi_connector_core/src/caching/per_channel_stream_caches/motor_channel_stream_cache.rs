use std::time::Instant;
use feagi_data_structures::{FeagiDataError, FeagiSignal, FeagiSignalIndex};
use feagi_data_structures::genomic::descriptors::CorticalChannelIndex;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPDecoder};
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::{PipelineStage, PipelineStageProperties, PipelineStagePropertyIndex, PipelineStageRunner};

#[derive(Debug)]
pub(crate) struct MotorChannelStreamCache {
    most_recent_directly_decoded_output: WrappedIOData,
    pipeline_runner: PipelineStageRunner,
    value_updated_callback: FeagiSignal<()>
}

// NOTE: We aim to generally abstract away [PipelineStageRunner] data operations from here onward

impl MotorChannelStreamCache {

    pub fn new(pipeline_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<Self, FeagiDataError> {

        let processor_runner = PipelineStageRunner::new(pipeline_stages)?;
        Ok(MotorChannelStreamCache {
            most_recent_directly_decoded_output: processor_runner.get_input_data_type().create_blank_data_of_type()?,
            pipeline_runner: processor_runner,
            value_updated_callback: FeagiSignal::new()
        })
    }

    //region Properties

    /// Returns the input data type expected by the processor chain.
    ///
    /// Indicates what type of data this cache expects to receive from sensors.
    /// This is determined by the first processor in the chain and enables
    /// type validation before data is fed into the cache.
    ///
    /// # Returns
    ///
    /// The `IOTypeVariant` that represents the expected input data type
    pub fn get_input_data_type(&self) -> WrappedIOType {
        self.pipeline_runner.get_input_data_type()
    }

    /// Returns the output data type produced by the processor chain.
    ///
    /// Indicates what type of data this cache produces after processing.
    /// This is determined by the final processor in the chain and enables
    /// consumers to understand the format of processed data.
    ///
    /// # Returns
    ///
    /// The `IOTypeVariant` that represents the output data type
    pub fn get_output_data_type(&self) -> WrappedIOType {
        self.pipeline_runner.get_output_data_type()
    }

    pub(crate) fn get_pipeline_runner(&self) -> &PipelineStageRunner {
        &self.pipeline_runner
    }

    pub(crate) fn get_pipeline_runner_mut(&mut self) -> &mut PipelineStageRunner {
        &mut self.pipeline_runner
    }

    //endregion

    //region Data


    pub fn get_most_recent_preprocessed_motor_value(&self) -> &WrappedIOData {
        &self.most_recent_directly_decoded_output
    }

    pub fn get_most_recent_postprocessed_motor_value(&self) -> &WrappedIOData {
        self.pipeline_runner.get_most_recent_output()
    }

    pub(crate) fn get_neuron_decode_data_location_ref_mut(&mut self) -> &mut WrappedIOData {
        &mut self.most_recent_directly_decoded_output
    }

    //endregion


    //region Callbacks

    pub fn connect_to_data_processed_signal<F>(&mut self, callback: F) -> FeagiSignalIndex 
    where
        F: Fn(&()) + Send + Sync + 'static,
    {
        self.value_updated_callback.connect(callback)
    }

    pub fn disconnect_to_data_processed_signal(&mut self, index: FeagiSignalIndex) -> Result<(), FeagiDataError> {
        self.value_updated_callback.disconnect(index)
    }

    //endregion


}