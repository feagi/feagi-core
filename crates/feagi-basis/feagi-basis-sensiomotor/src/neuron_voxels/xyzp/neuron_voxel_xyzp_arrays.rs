use super::NeuronVoxelXYZP;
use feagi_data::feagi_data_error::{FeagiDataError, FeagiFailDataEtc};

fn feagi_data_etc_error(message: String) -> FeagiDataError {
    let context: &'static str = Box::leak(message.into_boxed_str());
    FeagiFailDataEtc::new(context).into()
}

/// Structure-of-arrays storage for neuron voxel XYZP data.
///
/// Does not check for duplicate neuron coordinates automatically.
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelXYZPArrays {
    x: Vec<u32>,
    y: Vec<u32>,
    z: Vec<u32>,
    p: Vec<f32>,
}

/// Historical alias used throughout this crate for sparse XYZP neuron lists.
pub type NeuronVoxelXYZPSparseVectors = NeuronVoxelXYZPArrays;

impl NeuronVoxelXYZPArrays {
    pub fn new() -> Self {
        NeuronVoxelXYZPArrays {
            x: Vec::new(),
            y: Vec::new(),
            z: Vec::new(),
            p: Vec::new(),
        }
    }

    pub fn new_from_vectors(
        x: Vec<u32>,
        y: Vec<u32>,
        z: Vec<u32>,
        p: Vec<f32>,
    ) -> Result<Self, FeagiDataError> {
        let len = x.len();
        if len != y.len() || len != z.len() || len != p.len() {
            return Err(feagi_data_etc_error(
                "Input vectors must be the same length to generate XYZP neuron data!!".into(),
            ));
        }
        Ok(NeuronVoxelXYZPArrays { x, y, z, p })
    }

    pub fn with_capacity(number_of_neurons_initial: usize) -> Self {
        NeuronVoxelXYZPArrays {
            x: Vec::with_capacity(number_of_neurons_initial),
            y: Vec::with_capacity(number_of_neurons_initial),
            z: Vec::with_capacity(number_of_neurons_initial),
            p: Vec::with_capacity(number_of_neurons_initial),
        }
    }

    pub fn capacity(&self) -> usize {
        self.x.capacity()
    }

    pub fn len(&self) -> usize {
        self.p.len()
    }

    pub fn is_empty(&self) -> bool {
        self.p.is_empty()
    }

    pub fn ensure_capacity(&mut self, number_of_neurons_total: usize) {
        if self.capacity() >= number_of_neurons_total {
            return;
        }
        self.reserve(number_of_neurons_total - self.len());
    }

    pub fn reserve(&mut self, additional_neuron_count: usize) {
        self.x.reserve(additional_neuron_count);
        self.y.reserve(additional_neuron_count);
        self.z.reserve(additional_neuron_count);
        self.p.reserve(additional_neuron_count);
    }

    pub fn push(&mut self, neuron: &NeuronVoxelXYZP) {
        self.push_raw(
            neuron.neuron_voxel_coordinate.x,
            neuron.neuron_voxel_coordinate.y,
            neuron.neuron_voxel_coordinate.z,
            neuron.potential,
        );
    }

    pub fn push_raw(&mut self, x: u32, y: u32, z: u32, p: f32) {
        self.x.push(x);
        self.y.push(y);
        self.z.push(z);
        self.p.push(p);
    }

    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
        self.z.clear();
        self.p.clear();
    }

    pub fn get_size_in_number_of_bytes(&self) -> usize {
        self.len() * NeuronVoxelXYZP::NUMBER_BYTES_PER_NEURON
    }

    pub fn borrow_xyzp_vectors(&self) -> (&Vec<u32>, &Vec<u32>, &Vec<u32>, &Vec<f32>) {
        (&self.x, &self.y, &self.z, &self.p)
    }

    pub fn iter(&self) -> impl Iterator<Item = NeuronVoxelXYZP> + '_ {
        self.x
            .iter()
            .zip(self.y.iter())
            .zip(self.z.iter())
            .zip(self.p.iter())
            .map(|(((x, y), z), p)| NeuronVoxelXYZP::new(*x, *y, *z, *p))
    }

    pub fn update_vectors_from_external<F>(
        &mut self,
        vectors_changer: F,
    ) -> Result<(), FeagiDataError>
    where
        F: FnOnce(
            &mut Vec<u32>,
            &mut Vec<u32>,
            &mut Vec<u32>,
            &mut Vec<f32>,
        ) -> Result<(), FeagiDataError>,
    {
        vectors_changer(&mut self.x, &mut self.y, &mut self.z, &mut self.p)?;
        self.verify_equal_vector_lengths()
    }

    fn verify_equal_vector_lengths(&self) -> Result<(), FeagiDataError> {
        let len = self.x.len();
        if self.y.len() != len || self.z.len() != len || self.p.len() != len {
            return Err(feagi_data_etc_error(
                "Internal XYZP Arrays do not have equal lengths!".into(),
            ));
        }
        Ok(())
    }
}

impl Default for NeuronVoxelXYZPArrays {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NeuronVoxelXYZPArrays {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "NeuronVoxelXYZPArrays(X: {:?}, Y: {:?}, Z: {:?}, P: {:?})",
            self.x, self.y, self.z, self.p
        )
    }
}
