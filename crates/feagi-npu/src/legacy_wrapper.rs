use ahash::HashMap;


pub struct NPULegacyBurstEngineWrapper {
    engine: BurstEngineCpuRayon<FeagiStandardModelStandard32BitQuant>
}

impl NPULegacyBurstEngineWrapper {

    pub fn new() -> Self {
        Self {
            engine: BurstEngineCpuRayon::new() // TODO init function must be externally usable
        }
    }

    pub fn run_burst(&mut self) -> (BurstIndex<u32>, HashMap<CorticalID, BitPackedVector<u32>>) {
        // TODO All this needs to do is call the correct burst phases. THe example can be followed
        // but also include the phase of consolidating the fired neurons otherwise the result will always be
        // empty.
    }
}