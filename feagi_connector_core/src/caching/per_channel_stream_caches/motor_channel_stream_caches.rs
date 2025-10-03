use std::time::Instant;
use feagi_data_structures::{FeagiDataError, FeagiSignal, FeagiSignalIndex};
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex};
use feagi_data_structures::neurons::xyzp::CorticalMappedXYZPNeuronData;
use crate::data_pipeline::{PipelineStageProperties, PipelineStageRunner};
use crate::neuron_coding::xyzp::NeuronXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub(crate) struct MotorChannelStreamCaches {
    neuron_decoder: Box<dyn NeuronXYZPDecoder>,
    pipeline_runners: Vec<PipelineStageRunner>,
    most_recent_directly_decoded_outputs: Vec<WrappedIOData>,
    value_updated_callbacks: Vec<FeagiSignal<()>>,
}

impl MotorChannelStreamCaches {
    pub fn new(neuron_decoder: Box<dyn NeuronXYZPDecoder>, stage_properties_per_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>) -> Result<Self, FeagiDataError> {
        if stage_properties_per_channels.is_empty() {
            return Err(FeagiDataError::InternalError("MotorChannelStreamCaches Cannot be initialized with 0 channels!".into()))
        }

        let num_channels = stage_properties_per_channels.len();
        let mut pipeline_runners: Vec<PipelineStageRunner> = Vec::with_capacity(num_channels);
        let mut most_recent_directly_decoded_outputs: Vec<WrappedIOData> = Vec::with_capacity(num_channels);
        let mut callbacks: Vec<FeagiSignal<()>> = Vec::with_capacity(num_channels);

        for stage_properties_per_channel in stage_properties_per_channels {
            let pipeline_runner = PipelineStageRunner::new(stage_properties_per_channel)?;
            let blank_data = pipeline_runner.get_input_data_type().create_blank_data_of_type()?;
            callbacks.push(FeagiSignal::new());
            
            pipeline_runners.push(pipeline_runner);
            most_recent_directly_decoded_outputs.push(blank_data);
        }

        Ok(Self {
            neuron_decoder,
            pipeline_runners,
            most_recent_directly_decoded_outputs,
            value_updated_callbacks: callbacks,
        })
    }

    //region Properties

    pub fn number_of_channels(&self) -> CorticalChannelCount {
        (self.pipeline_runners.len() as u32).into()
    }

    //endregion

    //region Pipeline Runner Data

    pub fn try_get_channel_output_type(&self, cortical_channel_index: CorticalChannelIndex) -> Result<WrappedIOType, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_output_data_type())
    }

    pub fn try_get_most_recent_preprocessed_motor_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        _ = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(&self.most_recent_directly_decoded_outputs[*cortical_channel_index as usize])
    }

    pub fn try_get_most_recent_postprocessed_motor_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_most_recent_output())
    }

    pub fn try_get_channel_last_processed_instant(&self, cortical_channel_index: CorticalChannelIndex) -> Result<Instant, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_last_processed_instant())
    }

    /*
    pub(crate) fn try_get_neuron_decode_data_location_ref_mut(&mut self, cortical_channel_index: CorticalChannelIndex) -> Result<&mut WrappedIOData, FeagiDataError> {
        match self.most_recent_directly_decoded_outputs.get_mut(*cortical_channel_index as usize) {
            Some(data) => Ok(data),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of bounds for SensoryChannelStreamCaches with {} channels!",
                                                              cortical_channel_index, self.pipeline_runners.len())))
        }


    }

     */

    //endregion

    //region Callbacks

    pub fn try_connect_to_data_processed_signal<F>(&mut self, cortical_channel_index: CorticalChannelIndex, callback: F) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&()) + Send + Sync + 'static,
    {
        _ = self.try_get_pipeline_runner(cortical_channel_index)?;
        let idx = *cortical_channel_index as usize;
        Ok(self.value_updated_callbacks[idx].connect(callback))
    }

    pub fn try_disconnect_to_data_processed_signal(&mut self, cortical_channel_index: CorticalChannelIndex, signal_index: FeagiSignalIndex) -> Result<(), FeagiDataError> {
        _ = self.try_get_pipeline_runner(cortical_channel_index)?;
        let idx = *cortical_channel_index as usize;
        self.value_updated_callbacks[idx].disconnect(signal_index)
    }

    //endregion

    pub(crate) fn try_read_neuron_datato_wrapped_io_data(&mut self, neuron_data: &CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        self.neuron_decoder.read_neuron_data_multi_channel(&self.pipeline_runners)
    }

    //region Internal

    #[inline]
    fn try_get_pipeline_runner(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&PipelineStageRunner, FeagiDataError> {
        match self.pipeline_runners.get(*cortical_channel_index as usize) {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of bounds for SensoryChannelStreamCaches with {} channels!",
                                                              cortical_channel_index, self.pipeline_runners.len())))
        }

    }

    #[inline]
    fn try_get_pipeline_runner_mut(&mut self, cortical_channel_index: CorticalChannelIndex) -> Result<&mut PipelineStageRunner, FeagiDataError> {
        let num_runners = self.pipeline_runners.len();
        match self.pipeline_runners.get_mut(*cortical_channel_index as usize) {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of bounds for SensoryChannelStreamCaches with {} channels!",
                                                              cortical_channel_index, num_runners)))
        }

    }

    //endregion

}