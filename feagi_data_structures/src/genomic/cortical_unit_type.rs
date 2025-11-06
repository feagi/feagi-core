use crate::genomic::cortical_area::io_cortical_area_data_type::{FrameChangeHandling, PercentageNeuronPositioning};

// TODO this should be macro generated from template!
pub enum SensoryCorticalUnit {
    Infrared( (FrameChangeHandling, PercentageNeuronPositioning), ),
    Vision( (FrameChangeHandling, ), ),
    SegmentedVision( (FrameChangeHandling, ), (FrameChangeHandling, ), (FrameChangeHandling, ),
                      (FrameChangeHandling, ), (FrameChangeHandling, ), (FrameChangeHandling, ),
                      (FrameChangeHandling, ), (FrameChangeHandling, ), (FrameChangeHandling, ), ),
}

// TODO talk about experience of user of switching from one type to the other

fn test() {
    let a = SensoryCorticalUnit::SegmentedVision()
}