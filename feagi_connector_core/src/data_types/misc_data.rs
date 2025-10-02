use ndarray::{s, Array3, Zip};
use feagi_data_structures::FeagiDataError;
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
#[derive(Clone, Debug)]
pub struct MiscData {
    data: Array3<f32>
}

impl MiscData {

    //region Common Constructors

    /// Creates a new MiscData with zero-filled data of the specified dimensions.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::{MiscData, descriptors::MiscDataDimensions};
    /// 
    /// let dims = MiscDataDimensions::new(5, 5, 3).unwrap();
    /// let misc_data = MiscData::new(&dims).unwrap();
    /// assert_eq!(misc_data.get_internal_data().shape(), &[5, 5, 3]);
    /// ```
    pub fn new(resolution: &MiscDataDimensions) -> Result<MiscData, FeagiDataError> {
        Ok(MiscData {
            data: Array3::zeros([resolution.width as usize, resolution.height as usize, resolution.depth as usize])
        })
    }

    /// Creates a MiscData from an existing 3D array.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::MiscData;
    /// use ndarray::Array3;
    /// 
    /// let data = Array3::zeros((4, 4, 2));
    /// let misc_data = MiscData::new_with_data(data).unwrap();
    /// assert_eq!(misc_data.get_dimensions().depth, 2);
    /// ```
    pub fn new_with_data(data: Array3<f32>) -> Result<MiscData, FeagiDataError> {
        let shape = data.shape();
        if shape[0] == 0 || shape[1] == 0 || shape[2] == 0 {
            return Err(FeagiDataError::BadParameters("Misc Data cannot be empty!".into()));
        }
        Ok(MiscData{data})
    }

    /// Creates MiscData from an ImageFrame, converting u8 pixels to normalized f32 values.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::{ImageFrame, MiscData, descriptors::*};
    /// 
    /// let props = ImageFrameProperties::new(
    ///     ImageXYResolution::new(32, 32).unwrap(),
    ///     ColorSpace::Gamma,
    ///     ColorChannelLayout::RGB
    /// ).unwrap();
    /// let image = ImageFrame::new_from_image_frame_properties(&props).unwrap();
    /// let misc_data = MiscData::new_from_image_frame(&image).unwrap();
    /// assert_eq!(misc_data.get_dimensions().width, 32);
    /// ```
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

    /// Creates a 1x1x1 MiscData containing a single float value.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::MiscData;
    /// 
    /// let misc_data = MiscData::new_from_f32(42.5).unwrap();
    /// assert_eq!(misc_data.get_internal_data()[(0, 0, 0)], 42.5);
    /// ```
    pub fn new_from_f32(value: f32) -> Result<MiscData, FeagiDataError> {
        let mut output = MiscData::new(&MiscDataDimensions::new(1, 1, 1)?)?;
        let output_data = output.get_internal_data_mut();
        output_data[(0, 0, 0)] = value;
        Ok(output)
    }

    // TODO multi dimensional f32


    //endregion

    //region Get Properties

    /// Returns the dimensions of the data array.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::{MiscData, descriptors::MiscDataDimensions};
    /// 
    /// let dims = MiscDataDimensions::new(8, 6, 4).unwrap();
    /// let misc_data = MiscData::new(&dims).unwrap();
    /// let retrieved_dims = misc_data.get_dimensions();
    /// assert_eq!(retrieved_dims.width, 8);
    /// assert_eq!(retrieved_dims.height, 6);
    /// assert_eq!(retrieved_dims.depth, 4);
    /// ```
    pub fn get_dimensions(&self) -> MiscDataDimensions {
        MiscDataDimensions::new(
            self.data.shape()[0] as u32,
            self.data.shape()[1] as u32,
            self.data.shape()[2] as u32,
        ).unwrap()
    }

    //endregion

    /// Returns a reference to the internal 3D array.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::MiscData;
    /// 
    /// let misc_data = MiscData::new_from_f32(3.14).unwrap();
    /// let data = misc_data.get_internal_data();
    /// assert_eq!(data[(0, 0, 0)], 3.14);
    /// ```
    pub fn get_internal_data(&self) -> &Array3<f32> {
        &self.data
    }

    /// Returns a mutable reference to the internal 3D array.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::MiscData;
    /// 
    /// let mut misc_data = MiscData::new_from_f32(1.0).unwrap();
    /// let data = misc_data.get_internal_data_mut();
    /// data[(0, 0, 0)] = 2.5;
    /// assert_eq!(misc_data.get_internal_data()[(0, 0, 0)], 2.5);
    /// ```
    pub fn get_internal_data_mut(&mut self) -> &mut Array3<f32> {
        &mut self.data
    }
    
    pub fn blank_data(&mut self) {
        self.data.fill(0.0);
    }
}

impl std::fmt::Display for MiscData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MiscData({:?})", self.get_dimensions())
    }
}