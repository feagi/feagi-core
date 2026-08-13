use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine_enum::ComposableBurstEngineEnum;


/// This whole struct should be under npu under another thread so this one can loop without interruption


pub(crate) struct BurstEngineHandler<FIQ: FeagiIndexQuantization> {


    _p: PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> BurstEngineHandler<FIQ>
{






    fn run_burst_engines(&mut self, engines: ()) {



        





    }



}



unsafe impl<FIQ: FeagiIndexQuantization> Sync for BurstEngineHandler<FIQ> {}