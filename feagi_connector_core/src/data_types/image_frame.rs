
use ndarray::{Array3, ArrayView3, ArrayViewMut3, Zip};
use image::{DynamicImage, GenericImageView};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::genomic::descriptors::CorticalChannelIndex;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays};
use super::descriptors::{ColorChannelLayout, ColorSpace, MemoryOrderLayout, ImageFrameProperties, ImageXYResolution, ImageXYZDimensions};




// Named constants for sRGB / linear conversions
const SRGB_THRESHOLD: f32 = 0.04045;
const LINEAR_THRESHOLD: f32 = 0.0031308;
const SRGB_A: f32 = 1.055;
const SRGB_B: f32 = 0.055;
const LINEAR_SCALE: f32 = 12.92;
const GAMMA_MIDPOINT: f32 = 128.0;
const LINEAR_MIDPOINT: f32 = 0.5;

/// A container for image data with support for various color formats and spaces.
/// 
/// Stores pixel data as a 3D array with height, width, and channel dimensions.
/// Supports RGB/RGBA formats and different color spaces (sRGB, Linear, Gamma).
/// Can import/export various image formats and convert between color spaces.
#[derive(Clone, Debug)]
pub struct ImageFrame {
    pixels: Array3<u8>, // MemoryOrderLayout::HeightsWidthsChannels
    channel_layout: ColorChannelLayout,
    color_space: ColorSpace,
    /// If true, tells encoders to not encode this image, to instead send "blank"
    pub skip_encoding: bool,
}

// NOTE -> (0,0) is in the top left corner!

impl ImageFrame {
    /// The internal memory layout used for storing pixel data
    pub const INTERNAL_MEMORY_LAYOUT: MemoryOrderLayout = MemoryOrderLayout::HeightsWidthsChannels;



    //region Common Constructors

    /// Creates a new ImageFrame with zero-filled pixel data.
    pub fn new(channel_format: &ColorChannelLayout, color_space: &ColorSpace, xy_resolution: &ImageXYResolution) -> Result<ImageFrame, FeagiDataError> {
        Ok(ImageFrame {
            channel_layout: *channel_format,
            color_space: *color_space,
            pixels: Array3::<u8>::zeros((xy_resolution.height as usize, xy_resolution.width as usize, *channel_format as usize)),
            skip_encoding: false,
        })
    }

    /// Creates a new ImageFrame from ImageFrameProperties.
    pub fn new_from_image_frame_properties(image_frame_properties: &ImageFrameProperties) -> Result<ImageFrame, FeagiDataError>
    {
        ImageFrame::new(&image_frame_properties.get_color_channel_layout(), &image_frame_properties.get_color_space(), &image_frame_properties.get_image_resolution())
    }

    /// Creates an ImageFrame from a 3D array with specified memory layout.
    pub fn from_array(input: Array3<u8>, color_space: &ColorSpace, source_memory_order: &MemoryOrderLayout) -> Result<ImageFrame, FeagiDataError> {
        let pixel_data =  change_memory_order_to_row_major(input, source_memory_order);
        let number_color_channels: usize = pixel_data.shape()[2];
        Ok(ImageFrame {
            pixels: pixel_data,
            color_space: *color_space,
            channel_layout: ColorChannelLayout::try_from(number_color_channels)?,
            skip_encoding: false,
        })
    }

    pub fn new_from_dynamic_image(img: DynamicImage, color_space: &ColorSpace) -> Result<ImageFrame, FeagiDataError> {
        let (width, height) = img.dimensions();
        let color_layout = ColorChannelLayout::try_from(img.color())?;
        match color_layout {
            ColorChannelLayout::GrayScale => {
                let buffer = img.to_luma8();
                let array = Array3::from_shape_vec(
                    (height as usize, width as usize, 1),
                    buffer.into_raw()).unwrap();
                Self::from_array(array, color_space, &MemoryOrderLayout::HeightsWidthsChannels)
            },
            ColorChannelLayout::RG => {
                let buffer = img.to_luma_alpha8();
                let array = Array3::from_shape_vec(
                    (height as usize, width as usize, 2),
                    buffer.into_raw()).unwrap();
                Self::from_array(array, color_space, &MemoryOrderLayout::HeightsWidthsChannels)
            }
            ColorChannelLayout::RGB => {
                let buffer = img.to_rgb8();
                let array = Array3::from_shape_vec(
                    (height as usize, width as usize, 3),
                    buffer.into_raw()).unwrap();
                Self::from_array(array, color_space, &MemoryOrderLayout::HeightsWidthsChannels)
            }
            ColorChannelLayout::RGBA => {
                let buffer = img.to_rgba8();
                let array = Array3::from_shape_vec(
                    (height as usize, width as usize, 4),
                    buffer.into_raw()).unwrap();
                Self::from_array(array, color_space, &MemoryOrderLayout::HeightsWidthsChannels)
            }
        }
    }

    pub fn new_from_png_bytes(input: &[u8], color_space: &ColorSpace) -> Result<ImageFrame, FeagiDataError> {
        let image_format = image::ImageFormat::Png;
        let img = image::load_from_memory_with_format(input, image_format).unwrap();
        Self::new_from_dynamic_image(img, color_space)
    }

    pub fn new_from_bmp_bytes(input: &[u8], color_space: &ColorSpace) -> Result<ImageFrame, FeagiDataError> {
        let image_format = image::ImageFormat::Bmp;
        let img = image::load_from_memory_with_format(input, image_format).unwrap();
        Self::new_from_dynamic_image(img, color_space)
    }

    pub fn new_from_jpeg_bytes(input: &[u8], color_space: &ColorSpace) -> Result<ImageFrame, FeagiDataError> {
        let image_format = image::ImageFormat::Jpeg;
        let img = image::load_from_memory_with_format(input, image_format).unwrap();
        Self::new_from_dynamic_image(img, color_space)
    }

    pub fn new_from_tiff_bytes(input: &[u8], color_space: &ColorSpace) -> Result<ImageFrame, FeagiDataError> {
        let image_format = image::ImageFormat::Tiff;
        let img = image::load_from_memory_with_format(input, image_format).unwrap();
        Self::new_from_dynamic_image(img, color_space)
    }

    //endregion



    //region Properties

    /// Returns the properties of this image frame.
    ///
    /// Creates an ImageFrameProperties struct that describes this frame's
    /// resolution, color space, and channel layout.
    ///
    /// # Returns
    ///
    /// An ImageFrameProperties struct containing this frame's properties.
    pub fn get_image_frame_properties(&self) -> ImageFrameProperties {
        ImageFrameProperties::new(
            self.get_xy_resolution(),
            self.color_space,
            self.channel_layout
        ).unwrap()
    }

    /// Returns a reference to the channel layout of this image.
    ///
    /// # Returns
    ///
    /// A reference to the ChannelLayout enum value representing the image's color channel format.
    pub fn get_channel_layout(&self) -> &ColorChannelLayout {
        &self.channel_layout
    }

    /// Returns a reference to the color space of this image.
    ///
    /// # Returns
    ///
    /// A reference to the ColorSpace enum value representing the image's color space.
    pub fn get_color_space(&self) -> &ColorSpace {
        &self.color_space
    }

    /// Returns the number of color channels in this ImageFrame.
    ///
    /// # Returns
    ///
    /// The number of color channels as an usize:
    /// - 1 for GrayScale
    /// - 2 for RG
    /// - 3 for RGB
    /// - 4 for RGBA
    pub fn get_color_channel_count(&self) -> usize {
        self.channel_layout as usize
    }

    /// Returns a read-only view of the pixel data.
    ///
    /// This provides access to the underlying 3D ndarray of pixel values.
    ///
    /// # Returns
    ///
    /// An ArrayView3<f32> containing the pixel data.
    pub fn get_pixels_view(&self) -> ArrayView3<u8> {
        self.pixels.view()
    }
    
    /// Returns a mutable view of the pixel data as a 3D array.
    pub fn get_pixels_view_mut(&mut self) -> ArrayViewMut3<u8> {
        self.pixels.view_mut()
    }

    /// Returns the resolution of the image in cartesian space (width, height)
    ///
    /// # Returns
    ///
    /// A tuple of (width, height) representing the image dimensions in pixels.
    pub fn get_xy_resolution(&self) -> ImageXYResolution {
        let shape: &[usize] = self.pixels.shape();
        ImageXYResolution::new(shape[1] as u32, shape[0] as u32).unwrap() // because nd array is row major, where coords are yx
    }

    /// Returns the total number of elements (height × width × channels).
    pub fn get_number_elements(&self) -> usize {
        self.pixels.shape()[0] * self.pixels.shape()[1] * self.pixels.shape()[2]
    }

    /// Returns the 3D dimensions (height, width, channels) of the image.
    pub fn get_dimensions(&self) -> ImageXYZDimensions {
        ImageXYZDimensions::new(
            self.pixels.shape()[0] as u32,
            self.pixels.shape()[1] as u32,
            self.channel_layout.into()
        ).unwrap()
    }

    /// Returns a reference to the internal pixel data array.
    ///
    /// Provides direct access to the underlying 3D array containing the pixel data.
    /// The array is organized as (height, width, channels) following row-major ordering.
    ///
    /// # Returns
    ///
    /// A reference to the Array3<f32> containing the raw pixel data.
    ///
    /// # Safety
    ///
    /// This method provides direct access to internal data. Modifying the array
    /// through this reference could break invariants. Use `get_internal_data_mut()`
    /// for safe mutable access.
    pub fn get_internal_data(&self) -> &Array3<u8> {
        &self.pixels
    }

    /// Returns a mutable reference to the internal pixel data array.
    ///
    /// Provides mutable access to the underlying 3D array containing the pixel data.
    /// Be cautious when using this as you can easily set the data to an invalid state!
    ///
    /// # Returns
    ///
    /// A mutable reference to the Array3<f32> containing the raw pixel data.
    pub fn get_internal_data_mut(&mut self) -> &mut Array3<u8> {
        &mut self.pixels
    }

    pub fn get_internal_byte_data(&self) -> &[u8] {
        self.pixels.as_slice().unwrap()
    }

    pub fn get_internal_byte_data_mut(&mut self) -> &mut [u8] {
        self.pixels.as_slice_mut().unwrap()
    }
    
    //endregion



    //region Export as Image

    /// Exports the ImageFrame as a DynamicImage from the image crate.
    ///
    /// This converts the internal pixel data back to a format that can be
    /// used with the image crate for further processing or saving.
    ///
    /// # Returns
    ///
    /// A DynamicImage containing the pixel data from this ImageFrame.
    pub fn export_as_dynamic_image(&self) -> Result<DynamicImage, FeagiDataError> {
        let (width, height) = (self.get_xy_resolution().width as usize, self.get_xy_resolution().height as usize);
        
        match self.channel_layout {
            ColorChannelLayout::GrayScale => {
                let mut buffer = Vec::with_capacity(width * height);
                for y in 0..height {
                    for x in 0..width {
                        buffer.push(self.pixels[(y, x, 0)]);
                    }
                }
                let img_buffer = image::GrayImage::from_raw(width as u32, height as u32, buffer)
                    .ok_or_else(|| FeagiDataError::InternalError("Failed to create grayscale image".to_string()))?;
                Ok(DynamicImage::ImageLuma8(img_buffer))
            },
            ColorChannelLayout::RG => {
                let mut buffer = Vec::with_capacity(width * height * 2);
                for y in 0..height {
                    for x in 0..width {
                        buffer.push(self.pixels[(y, x, 0)]); // L
                        buffer.push(self.pixels[(y, x, 1)]); // A
                    }
                }
                let img_buffer = image::GrayAlphaImage::from_raw(width as u32, height as u32, buffer)
                    .ok_or_else(|| FeagiDataError::InternalError("Failed to create grayscale+alpha image".to_string()))?;
                Ok(DynamicImage::ImageLumaA8(img_buffer))
            },
            ColorChannelLayout::RGB => {
                let mut buffer = Vec::with_capacity(width * height * 3);
                for y in 0..height {
                    for x in 0..width {
                        buffer.push(self.pixels[(y, x, 0)]); // R
                        buffer.push(self.pixels[(y, x, 1)]); // G
                        buffer.push(self.pixels[(y, x, 2)]); // B
                    }
                }
                let img_buffer = image::RgbImage::from_raw(width as u32, height as u32, buffer)
                    .ok_or_else(|| FeagiDataError::InternalError("Failed to create RGB image".to_string()))?;
                Ok(DynamicImage::ImageRgb8(img_buffer))
            },
            ColorChannelLayout::RGBA => {
                let mut buffer = Vec::with_capacity(width * height * 4);
                for y in 0..height {
                    for x in 0..width {
                        buffer.push(self.pixels[(y, x, 0)]); // R
                        buffer.push(self.pixels[(y, x, 1)]); // G
                        buffer.push(self.pixels[(y, x, 2)]); // B
                        buffer.push(self.pixels[(y, x, 3)]); // A
                    }
                }
                let img_buffer = image::RgbaImage::from_raw(width as u32, height as u32, buffer)
                    .ok_or_else(|| FeagiDataError::InternalError("Failed to create RGBA image".to_string()))?;
                Ok(DynamicImage::ImageRgba8(img_buffer))
            }
        }
    }

    /// Exports the ImageFrame as PNG bytes.
    ///
    /// # Returns
    ///
    /// A Vec<u8> containing the PNG-encoded image data.
    pub fn export_as_png_bytes(&self) -> Result<Vec<u8>, FeagiDataError> {
        let dynamic_img = self.export_as_dynamic_image()?;
        let mut buffer = Vec::new();
        dynamic_img.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to encode PNG: {}", e)))?;
        Ok(buffer)
    }

    /// Exports the ImageFrame as BMP bytes.
    ///
    /// # Returns
    ///
    /// A Vec<u8> containing the BMP-encoded image data.
    pub fn export_as_bmp_bytes(&self) -> Result<Vec<u8>, FeagiDataError> {
        let dynamic_img = self.export_as_dynamic_image()?;
        let mut buffer = Vec::new();
        dynamic_img.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Bmp)
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to encode BMP: {}", e)))?;
        Ok(buffer)
    }

    /// Exports the ImageFrame as JPEG bytes.
    ///
    /// Note: JPEG format does not support transparency, so RGBA images will be
    /// converted to RGB by discarding the alpha channel.
    ///
    /// # Returns
    ///
    /// A Vec<u8> containing the JPEG-encoded image data.
    pub fn export_as_jpeg_bytes(&self) -> Result<Vec<u8>, FeagiDataError> {
        let mut dynamic_img = self.export_as_dynamic_image()?;
        
        // JPEG doesn't support transparency, convert RGBA to RGB
        if matches!(self.channel_layout, ColorChannelLayout::RGBA) {
            dynamic_img = DynamicImage::ImageRgb8(dynamic_img.to_rgb8());
        }
        
        let mut buffer = Vec::new();
        dynamic_img.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Jpeg)
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to encode JPEG: {}", e)))?;
        Ok(buffer)
    }

    /// Exports the ImageFrame as TIFF bytes.
    ///
    /// # Returns
    ///
    /// A Vec<u8> containing the TIFF-encoded image data.
    pub fn export_as_tiff_bytes(&self) -> Result<Vec<u8>, FeagiDataError> {
        let dynamic_img = self.export_as_dynamic_image()?;
        let mut buffer = Vec::new();
        dynamic_img.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Tiff)
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to encode TIFF: {}", e)))?;
        Ok(buffer)
    }

    //endregion



    //region Image Processing

    //region In Place

    // TODO move these to be SIMD

    /// Apply in-place brightness adjustment
    /// `value` = signed integer added to each channel (only in gamma space)
    pub fn change_brightness(&mut self, value: i32) {
        match self.color_space {
            ColorSpace::Gamma => {

                Zip::indexed(&mut self.pixels).par_for_each(|(_y,_x,_c), color_val| {
                    let v = (*color_val as i32 + value).clamp(0, 255);
                    *color_val = v as u8;
                });
            }
            ColorSpace::Linear => {
                // Convert to linear float, add value scaled to 0-1, clamp
                let delta = value as f32 / 255.0;
                
                // TODO make parallel
                Zip::from(&mut self.pixels).for_each(|px| {
                    let lin = Self::srgb_to_linear(*px as f32);
                    let lin = (lin + delta).clamp(0.0, 1.0);
                    *px = Self::linear_to_srgb(lin);
                });
            }
        }
    }

    /// Apply in-place contrast adjustment
    /// `factor` = multiplier (>1 = increase contrast, <1 = reduce)
    pub fn change_contrast(&mut self, factor: f32) {
        match self.color_space {
            ColorSpace::Gamma => {
                // Contrast around mid-point 128
                Zip::from(&mut self.pixels).for_each(|px| {
                    let v = ((*px as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0);
                    *px = v.round() as u8;
                });
            }
            ColorSpace::Linear => {
                // Convert to linear float, adjust contrast around 0.5, then back
                Zip::from(&mut self.pixels).for_each(|px| {
                    let lin = Self::srgb_to_linear(*px as f32);
                    let lin = ((lin - 0.5) * factor + 0.5).clamp(0.0, 1.0);
                    *px = Self::linear_to_srgb(lin);
                });
            }
        }
    }

   pub fn blink_image(&mut self) {
       self.pixels.fill(0);
   }

    //endregion



    //region Internal Color Space Conversions

    /// Convert sRGB byte [0..255] -> linear [0.0..1.0]
    #[inline]
    fn srgb_to_linear(c: f32) -> f32 {
        let c = c / 255.0;
        if c <= SRGB_THRESHOLD {
            c / LINEAR_SCALE
        } else { ((c + SRGB_B) / SRGB_A).powf(2.4) }
    }

    /// Convert linear [0.0..1.0] -> sRGB byte [0..255]
    #[inline]
    fn linear_to_srgb(c: f32) -> u8 {
        let c = if c <= LINEAR_THRESHOLD {
            LINEAR_SCALE * c
        } else {
            SRGB_A * c.powf(1.0 / 2.4) - SRGB_B
        };
        (c.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    //endregion



    //endregion


    // region Outputting Neurons

    pub(crate) fn overwrite_neuron_data(&self, write_target: &mut NeuronXYZPArrays, channel_index: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        const EPSILON: u8 = 1; // avoid writing near zero vals

        let x_offset: u32 = *channel_index * self.get_xy_resolution().width;
        write_target.clear();
        
        if self.skip_encoding {
            return Ok(()) // Encoding is to be skipped
        }
        
        write_target.ensure_capacity(self.get_number_elements());

        write_target.update_vectors_from_external(|x_vec, y_vec, c_vec, p_vec| {
            for ((x, y, c), color_val) in self.pixels.indexed_iter() { // going from row major to cartesian
                if color_val > &EPSILON {
                    x_vec.push(x as u32 + x_offset);
                    y_vec.push(y as u32);  // flip y //TODO wheres the flip part????
                    c_vec.push(c as u32);
                    p_vec.push(*color_val as f32 / 255.0);
                }
            };
            Ok(())
        })


    }

    // endregion

}

impl std::fmt::Display for ImageFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ImageFrame({})", self.get_image_frame_properties())
    }
}

fn change_memory_order_to_row_major(input: Array3<u8>, source_memory_order: &MemoryOrderLayout) -> Array3<u8> {
    match source_memory_order {
        MemoryOrderLayout::HeightsWidthsChannels => input, // Nothing needed, we store in this format anyway
        MemoryOrderLayout::ChannelsHeightsWidths => input.permuted_axes([2, 0, 1]),
        MemoryOrderLayout::WidthsHeightsChannels => input.permuted_axes([1, 0, 2]),
        MemoryOrderLayout::HeightsChannelsWidths => input.permuted_axes([0, 2, 1]),
        MemoryOrderLayout::ChannelsWidthsHeights => input.permuted_axes([2, 1, 0]),
        MemoryOrderLayout::WidthsChannelsHeights => input.permuted_axes([1, 2, 0]),
    }
}