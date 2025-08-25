use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::WrappedIOType;
use crate::data_pipeline::stream_cache_processor_trait::StreamCacheStage;

pub(crate) fn verify_sensor_chain(cache_processors: &Vec<Box<dyn StreamCacheStage + Sync + Send>>) -> Result<(), FeagiDataError> {
    let number_of_processors = cache_processors.len();

    if number_of_processors == 0 {
        return Err(FeagiDataError::BadParameters("Processor Runner cannot have 0 Cache Processors!".into()).into())
    }

    // Ensure data can pass between processing
    for processer_index in 0..number_of_processors - 1  {
        let first = &cache_processors[processer_index];
        let second = &cache_processors[processer_index + 1];
        if first.get_output_data_type() != second.get_input_data_type() {
            return Err(FeagiDataError::BadParameters(format!("Given cache processor at index {} has output type {}, which does not match the input type of cache processor at index {} or type {}!",
                                                              processer_index, first.get_output_data_type(), processer_index + 1,  second.get_input_data_type()).into()).into());
        }
    };

    Ok(())
}

pub(crate) fn verify_sensor_chain_and_encoder(cache_processors: &Vec<Box<dyn StreamCacheStage + Sync + Send>>, type_accepted_by_encoder: &WrappedIOType)
                                              -> Result<(), FeagiDataError> {
    
    verify_sensor_chain(cache_processors)?;

    if &cache_processors.last().unwrap().get_output_data_type() != type_accepted_by_encoder {
        return Err(FeagiDataError::BadParameters(format!("Given cache processor at index {} has output type {}, which does not match the input type of the encoder taking type {}!",
                                                          cache_processors.len() - 1, cache_processors.last().unwrap().get_output_data_type(), type_accepted_by_encoder).into()).into());
    }

    Ok(())
}
