use core::ops::Range;
use core::marker::PhantomData;
use ahash::HashMap;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_npu_common::wrapped_values::{CorticalConnectomeIndex, NeuronEngineIndex};

pub struct EngineDataReader<FGQ: FeagiGlobalQuantization, DataType> {
    area_mappings: HashMap<
        CorticalConnectomeIndex<FGQ::CorticalAreaIndexCountQuant>,
        Range<NeuronEngineIndex<FGQ::NeuronIndexCountQuant>>
    >,
    _p: PhantomData<DataType>,
}

impl<FGQ: FeagiGlobalQuantization, DataType> EngineDataReader<FGQ, DataType> {

    // TODO read from non clone

    pub fn read_from_local_data(index: CorticalConnectomeIndex<FGQ::CorticalAreaIndexCountQuant>, ) -> &[DataType]
    {


    }

}

