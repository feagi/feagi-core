

pub struct NeuronID {
    id: u32,
}

impl NeuronID {
    pub fn new(id: u32) -> NeuronID {
        NeuronID {
            id
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }




}