use core::ops::Range;
use core::marker::PhantomData;
use ahash::HashMap;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::burst_index::{CorticalConnectomeIndex, NeuronEngineIndex};

pub struct EngineDataReader<FIQ: FeagiIndexQuantization, DataType> {
    area_mappings: HashMap<
        CorticalConnectomeIndex<FIQ::CorticalAreaIndexCountQuant>,
        Range<NeuronEngineIndex<FIQ::NeuronIndexCountQuant>>
    >,
    _p: PhantomData<DataType>,
}

impl<FIQ: FeagiIndexQuantization, DataType> EngineDataReader<FIQ, DataType> {

    // TODO read from non clone
    
    pub fn read_from_local_data(index: CorticalConnectomeIndex<FIQ::CorticalAreaIndexCountQuant>, ) -> &[DataType]
    {


    }

}

