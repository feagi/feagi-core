use ndarray::{s, Array3, Zip};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::CorticalChannelIndex;
use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
use super::descriptors::MiscDataDimensions;
use super::ImageFrame;

/// A 3D array container for miscellaneous floating-point data.
/// 
/// Used for storing arbitrary 3D data structures with width, height, and depth dimensions.
/// Supports conversion from image frames and individual float values.
/// 
/// # Example
/// ```
/// use feagi_connector_core::data_types::{MiscData, descriptors::MiscDataDimensions};
/// 
/// let dims = MiscDataDimensions::new(10, 10, 5).unwrap();
/// let misc_data = MiscData::new(&dims).unwrap();
/// assert_eq!(misc_data.get_dimensions().width, 10);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MiscData {
    data: Array3<f32>
}

impl MiscData {

    //region Common Constructors


    pub fn new(resolution: &MiscDataDimensions) -> Result<MiscData, FeagiDataError> {
        Ok(MiscData {
            data: Array3::zeros([resolution.width as usize, resolution.height as usize, resolution.depth as usize])
        })
    }


    pub fn new_with_data(data: Array3<f32>) -> Result<MiscData, FeagiDataError> {
        let shape = data.shape();
        if shape[0] == 0 || shape[1] == 0 || shape[2] == 0 {
            return Err(FeagiDataError::BadParameters("Misc Data cannot be empty!".into()));
        }
        Ok(MiscData{data})
    }


    pub fn new_from_image_frame(image: &ImageFrame) -> Result<MiscData, FeagiDataError> {
        let mut output  = MiscData::new(&image.get_dimensions().into())?;
        let output_data = output.get_internal_data_mut();
        Zip::from(image.get_internal_data())
            .and(output_data)
            .par_for_each(|&src, dst| {
                *dst = (src as f32) / (u8::MAX as f32);
            });
        Ok(output)
    }


    pub fn new_from_f32(value: f32) -> Result<MiscData, FeagiDataError> {
        let mut output = MiscData::new(&MiscDataDimensions::new(1, 1, 1)?)?;
        let output_data = output.get_internal_data_mut();
        output_data[(0, 0, 0)] = value;
        Ok(output)
    }

    // TODO multi dimensional f32


    //endregion

    //region Get Properties


    pub fn get_dimensions(&self) -> MiscDataDimensions {
        MiscDataDimensions::new(
            self.data.shape()[0] as u32,
            self.data.shape()[1] as u32,
            self.data.shape()[2] as u32,
        ).unwrap()
    }

    //endregion


    pub fn get_internal_data(&self) -> &Array3<f32> {
        &self.data
    }


    pub fn get_internal_data_mut(&mut self) -> &mut Array3<f32> {
        &mut self.data
    }
    
    pub fn blank_data(&mut self) {
        self.data.fill(0.0);
    }


    // region Outputting Neurons

    pub fn overwrite_neuron_data(&self, write_target: &mut NeuronVoxelXYZPArrays, x_channel_offset: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        const EPSILON: f32 = 0.0001; // avoid writing near zero vals

        let x_offset: u32 = *x_channel_offset * self.get_dimensions().width;
        write_target.clear();
        write_target.ensure_capacity(self.get_dimensions().number_elements() as usize);

        write_target.update_vectors_from_external(|x_vec, y_vec, z_vec, p_vec| {
            for ((x, y, c), val) in self.data.indexed_iter() { // going from row major to cartesian
                if val.abs() > EPSILON {
                    x_vec.push(x as u32 + x_offset);
                    y_vec.push(y as u32);  // flip y //TODO wheres the flip part????
                    z_vec.push(c as u32);
                    p_vec.push(val.clamp(-1.0, 1.0));
                }
            };
            Ok(())
        })


    }

    // endregion
}

impl std::fmt::Display for MiscData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MiscData({:?})", self.get_dimensions())
    }
}