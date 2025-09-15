use ndarray::{Array3, Zip};
use ndarray::parallel::prelude::*;
use crate::basic_components::Dimensions;
use crate::data::ImageFrame;
use crate::FeagiDataError;

#[derive(Clone, Debug)]
pub struct MiscData {
    data: Array3<f32>
}

impl MiscData {

    //region Common Constructors

    pub fn new(resolution: &Dimensions) -> Result<MiscData, FeagiDataError> {
        Ok(MiscData {
            data: Array3::zeros([resolution.x as usize, resolution.y as usize, resolution.z as usize])
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
        let mut output  = MiscData::new(&image.get_dimensions())?;
        let output_data = output.get_internal_data_mut();
        Zip::from(image.get_internal_data())
            .and(output_data)
            .par_for_each(|&src, dst| {
                *dst = (src as f32) / (u8::MAX as f32);
            });
        Ok(output)
    }

    pub fn new_from_f32(value: f32) -> Result<MiscData, FeagiDataError> {
        let mut output = MiscData::new(&Dimensions::new(1,1,1)?)?;
        let output_data = output.get_internal_data_mut();
        output_data[(0, 0, 0)] = value;
        Ok(output)
    }



    //endregion

    //region Get Properties

    pub fn get_dimensions(&self) -> Dimensions {
        Dimensions::new(
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





}