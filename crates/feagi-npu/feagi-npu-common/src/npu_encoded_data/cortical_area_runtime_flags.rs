
/// Settings of a cortical area that are modified by genome developers
#[derive(Clone, Copy)]
pub struct CorticalAreaRuntimeFlags(u8);

impl CorticalAreaRuntimeFlags {
    const BITMASK_CORTICAL_AREA_PAUSED: u8 = 1 << 0;
    const BITMASK_PSP_UNIFORMITY: u8 = 1 << 1;
    const BITMASK_MP_DRIVEN_PSP: u8 = 1 << 2;
    
    pub fn get_membrane_driven_psp(self) -> bool {
        self.0 & Self::BITMASK_MP_DRIVEN_PSP != 0
    }

    pub fn get_psp_uniformity(self) -> bool {
        self.0 & Self::BITMASK_PSP_UNIFORMITY != 0
    }

    pub fn get_cortical_area_paused(self) -> bool {
        self.0 & Self::BITMASK_CORTICAL_AREA_PAUSED != 0
    }
}