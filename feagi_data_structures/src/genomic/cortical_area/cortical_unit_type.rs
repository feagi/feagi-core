use crate::genomic::cortical_area::io_cortical_area_data_type_descriptors::{FrameChangeHandling, PercentageNeuronPositioning};

pub enum SensoryCorticalUnit {
    Infrared(( (FrameChangeHandling, PercentageNeuronPositioning) )),
    Vision(( (FrameChangeHandling, ) )),
    SegmentedVision((FrameChangeHandling, ), (FrameChangeHandling, ), (FrameChangeHandling, ),
                    (FrameChangeHandling, ), (FrameChangeHandling, ), (FrameChangeHandling, ),
                    (FrameChangeHandling, ), (FrameChangeHandling, ), (FrameChangeHandling, ))
}