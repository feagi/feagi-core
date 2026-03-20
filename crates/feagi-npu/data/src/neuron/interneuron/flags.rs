
/// Stores various interneuron boolean flags under a single byte (u8)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterneuronFlag(u8);

impl InterneuronFlag {

    pub const INVALID_FLAG: InterneuronFlag = InterneuronFlag(0);

    pub fn new_valid() -> Self {
        Self(0x01) // The first flag is the valid flag
    }

    pub fn is_valid(&self) -> bool {
        self & 0x01 != 0
    }

    // Other 7 bits are reserved for now
    //pub fn get_b(&self) -> bool { self & 0x02 != 0 }
    //pub fn get_c(&self) -> bool { self & 0x04 != 0 }
    //pub fn get_d(&self) -> bool { self & 0x08 != 0 }
    //pub fn get_e(&self) -> bool { self & 0x10 != 0 }
    //pub fn get_f(&self) -> bool { self & 0x20 != 0 }
    //pub fn get_g(&self) -> bool { self & 0x40 != 0 }
    //pub fn get_h(&self) -> bool { self & 0x80 != 0 }


    pub fn set_validity(&mut self, is_valid: bool) {
        if is_valid {
            self |= 0x01;
        } else {
            self &= 0xFE;
        }
    }

    /*

    pub fn set_b(&mut self, value: bool) {
        if value {
            self |= 0x02;
        } else {
            self &= 0xFD;
        }
    }

    pub fn set_c(&mut self, value: bool) {
        if value {
            self |= 0x04;
        } else {
            self &= 0xFB;
        }
    }


    pub fn set_d(&mut self, value: bool) {
        if value {
            self |= 0x08;
        } else {
            self &= 0xF7;
        }
    }

    pub fn set_e(&mut self, value: bool) {
        if value {
            self |= 0x10;
        } else {
            self &= 0xEF;
        }
    }

    pub fn set_f(&mut self, value: bool) {
        if value {
            self |= 0x20;
        } else {
            self &= 0xDF;
        }
    }

    pub fn set_g(&mut self, value: bool) {
        if value {
            self |= 0x40;
        } else {
            self &= 0xBF;
        }
    }

    pub fn set_h(&mut self, value: bool) {
        if value {
            self |= 0x80;
        } else {
            self &= 0x7F;
        }
    }
    */


    pub fn toggle_validity(&mut self) { self ^= 0x01; }

    //pub fn toggle_mp_charge_accumulation_enabled(&mut self) { self ^= 0x02; }
    //pub fn toggle_mp_driven_psp_enabled(&mut self) { self ^= 0x04; }
    //pub fn toggle_d(&mut self) { self ^= 0x08; }
    //pub fn toggle_e(&mut self) { self ^= 0x10; }
    //pub fn toggle_f(&mut self) { self ^= 0x20; }
    //pub fn toggle_g(&mut self) { self ^= 0x40; }
    //pub fn toggle_h(&mut self) { self ^= 0x80; }

}


/// Stores various interneuron cortical area boolean flags under a single byte (u8)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterneuronCorticalFlag(u8);

impl InterneuronCorticalFlag {

    pub const INVALID_FLAG: InterneuronCorticalFlag = InterneuronCorticalFlag(0);

    pub fn new_valid() -> Self {
        Self(0x01) // The first flag is the valid flag
    }

    pub fn is_valid(&self) -> bool {
        self & 0x01 != 0
    }

    pub fn is_mp_charge_accumulation_enabled(&self) -> bool {
        self & 0x02 != 0
    }

    pub fn is_mp_driven_psp_enabled(&self) -> bool {
        self & 0x04 != 0
    }

    // Other 6 bits are reserved for now
    //pub fn get_d(&self) -> bool { self & 0x08 != 0 }
    //pub fn get_e(&self) -> bool { self & 0x10 != 0 }
    //pub fn get_f(&self) -> bool { self & 0x20 != 0 }
    //pub fn get_g(&self) -> bool { self & 0x40 != 0 }
    //pub fn get_h(&self) -> bool { self & 0x80 != 0 }


    pub fn set_validity(&mut self, is_valid: bool) {
        if is_valid {
            self |= 0x01;
        } else {
            self &= 0xFE;
        }
    }

    pub fn set_mp_charge_accumulation_enabled(&mut self, is_mp_charge_accumulation_enabled: bool) {
        if is_mp_charge_accumulation_enabled {
            self |= 0x02;
        } else {
            self &= 0xFD;
        }
    }

    pub fn set_mp_driven_psp(&mut self, is_mp_driven_psp_enabled: bool) {
        if is_mp_driven_psp_enabled {
            self |= 0x04;
        } else {
            self &= 0xFB;
        }
    }

    /*


    pub fn set_d(&mut self, value: bool) {
        if value {
            self |= 0x08;
        } else {
            self &= 0xF7;
        }
    }

    pub fn set_e(&mut self, value: bool) {
        if value {
            self |= 0x10;
        } else {
            self &= 0xEF;
        }
    }

    pub fn set_f(&mut self, value: bool) {
        if value {
            self |= 0x20;
        } else {
            self &= 0xDF;
        }
    }

    pub fn set_g(&mut self, value: bool) {
        if value {
            self |= 0x40;
        } else {
            self &= 0xBF;
        }
    }

    pub fn set_h(&mut self, value: bool) {
        if value {
            self |= 0x80;
        } else {
            self &= 0x7F;
        }
    }
    */


    pub fn toggle_validity(&mut self) { self ^= 0x01; }
    pub fn toggle_mp_charge_accumulation_enabled(&mut self) { self ^= 0x02; }
    pub fn toggle_mp_driven_psp_enabled(&mut self) { self ^= 0x04; }


    //pub fn toggle_d(&mut self) { self ^= 0x08; }
    //pub fn toggle_e(&mut self) { self ^= 0x10; }
    //pub fn toggle_f(&mut self) { self ^= 0x20; }
    //pub fn toggle_g(&mut self) { self ^= 0x40; }
    //pub fn toggle_h(&mut self) { self ^= 0x80; }

}