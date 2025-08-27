use ndarray::{Array3, ArrayView3};
use crate::FeagiDataError;
use crate::data::image_descriptors::{ColorChannelLayout, ColorSpace, MemoryOrderLayout, ImageFrameProperties, ImageXYResolution};
use crate::basic_components::Dimensions;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::CorticalChannelIndex;
use crate::neurons::xyzp::CorticalMappedXYZPNeuronData;

/// Represents an image frame with pixel data and metadata for FEAGI vision processing.
///
/// An `ImageFrame` stores image data as a 3D array of f32 values along with information
/// about the color channel format and color space. The internal storage uses row-major
/// ordering (height, width, channels) for efficient processing.
#[derive(Clone, Debug)]
pub struct ImageFrame {
    /// The pixel data stored as a 3D array with dimensions (height, width, channels)
    pixels: Array3<f32>,
    /// The color channel format (GrayScale, RG, RGB, or RGBA)
    channel_layout: ColorChannelLayout,
    /// The color space (Linear or Gamma)
    color_space: ColorSpace,
}

impl ImageFrame {
    /// The internal memory layout used for storing pixel data
    pub const INTERNAL_MEMORY_LAYOUT: MemoryOrderLayout = MemoryOrderLayout::HeightsWidthsChannels;

    // region Common Constructors

    /// Creates a new ImageFrame with the specified channel format, color space, and resolution.
    ///
    /// # Arguments
    ///
    /// * `channel_format` - The color channel format for the image (GrayScale, RG, RGB, or RGBA)
    /// * `color_space` - The color space of the image (Linear or Gamma)
    /// * `xy_resolution` - The resolution of the image as a tuple of (width, height)
    ///
    /// # Returns
    ///
    /// A new ImageFrame instance with all pixels initialized to zero.
    pub fn new(channel_format: &ColorChannelLayout, color_space: &ColorSpace, xy_resolution: &ImageXYResolution) -> Result<ImageFrame, FeagiDataError> {
        Ok(ImageFrame {
            channel_layout: *channel_format,
            color_space: *color_space,
            pixels: Array3::<f32>::zeros((xy_resolution.height, xy_resolution.width, *channel_format as usize)),
        })
    }

    /// Creates an ImageFrame from an existing ndarray with the specified color space.
    ///
    /// # Arguments
    ///
    /// * `input` - A 3D array of f32 values representing the image pixels
    /// * `color_space` - The color space of the image (Linear or Gamma)
    /// * `source_memory_order` - The memory layout of the input array
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// - Ok(ImageFrame) if the input array has a valid number of color channels (1-4)
    /// - Err(DataProcessingError) if the number of color channels is invalid
    pub fn from_array(input: Array3<f32>, color_space: &ColorSpace, source_memory_order: &MemoryOrderLayout) -> Result<ImageFrame, FeagiDataError> {
        let number_color_channels: usize = input.shape()[2];
        Ok(ImageFrame {
            pixels: ImageFrame::change_memory_order_to_row_major(input, source_memory_order),
            color_space: *color_space,
            channel_layout: ColorChannelLayout::try_from(number_color_channels)?
        })
    }


    /// Creates a new ImageFrame from ImageFrameProperties specification.
    ///
    /// Creates a new ImageFrame with all pixels initialized to zero, using the
    /// resolution, color space, and channel layout specified in the properties.
    ///
    /// # Arguments
    ///
    /// * `image_frame_properties` - Properties specifying the desired image configuration
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// - Ok(ImageFrame) if the frame was created successfully
    /// - Err(FeagiDataError) if the properties specify invalid dimensions
    pub fn from_image_frame_properties(image_frame_properties: &ImageFrameProperties) -> Result<ImageFrame, FeagiDataError>
    {
        ImageFrame::new(&image_frame_properties.get_color_channel_layout(), &image_frame_properties.get_color_space(), &image_frame_properties.get_image_resolution())
    }

    // endregion
    
    // region Get Properties

    /// Returns true if two ImageFrames have the same channel count, resolution, and color space.
    ///
    /// # Arguments
    ///
    /// * `a` - First ImageFrame to compare
    /// * `b` - Second ImageFrame to compare
    ///
    /// # Returns
    ///
    /// True if both frames have identical channel count, resolution, and color space.
    pub fn do_resolutions_channel_depth_and_color_spaces_match(a: &ImageFrame, b: &ImageFrame) -> bool {
        a.get_color_channel_count() == b.get_color_channel_count() && a.color_space == b.color_space
    }

    /// Returns true if the given array has valid dimensions for an ImageFrame.
    ///
    /// An array is considered valid if:
    /// - It has between 1 and 4 color channels
    /// - It has non-zero width and height
    ///
    /// # Arguments
    ///
    /// * `array` - The array to validate
    ///
    /// # Returns
    ///
    /// True if the array dimensions are valid for an ImageFrame, false otherwise.
    pub fn is_array_valid_for_image_frame(array: &Array3<f32>) -> bool {
        let shape: &[usize] = array.shape();
        if shape[2] > 4 || shape[2] == 0 {
            return false;
        }
        if shape[0] == 0 || shape[1] == 0 {
            return false;
        }
        true
    }

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
    pub fn get_pixels_view(&self) -> ArrayView3<f32> {
        self.pixels.view()
    }

    /// Returns the resolution of the image in cartesian space (width, height)
    ///
    /// # Returns
    ///
    /// A tuple of (width, height) representing the image dimensions in pixels.
    pub fn get_xy_resolution(&self) -> ImageXYResolution {
        let shape: &[usize] = self.pixels.shape();
        ImageXYResolution::new(shape[1], shape[0]).unwrap() // because nd array is row major, where coords are yx
    }

    /// Returns the internal shape of the image array in row-major order.
    ///
    /// The shape is returned as (height, width, channels) representing the dimensions
    /// of the internal ndarray storage.
    ///
    /// # Returns
    ///
    /// A tuple of (height, width, channels) representing the array dimensions.
    pub fn get_xyz_shape(&self) ->  Dimensions {
        let shape: &[usize] = self.pixels.shape();
        Dimensions::new(shape[1] as u32, shape[0] as u32, shape[2] as u32).unwrap()
    }
    
    pub fn get_number_elements(&self) -> usize {
        self.pixels.shape()[0] * self.pixels.shape()[1] * self.pixels.shape()[2]
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
    pub fn get_internal_data(&self) -> &Array3<f32> {
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
    pub fn get_internal_data_mut(&mut self) -> &mut Array3<f32> {
        &mut self.pixels
    }

    //endregion
    
    //region Image Processing

    //region In-Place
    
    pub fn change_brightness(&mut self, brightness_factor: f32) -> Result<(), FeagiDataError> { // TODO This algorithm is likely wrong given linear and gamma color spaces!
        if brightness_factor < 0.0 {
            return Err(FeagiDataError::BadParameters("Multiply brightness by must be positive!".into()).into());
        }

        self.pixels.mapv_inplace(|v| {
            let scaled = (v) * brightness_factor;
            scaled
            //scaled.clamp(0.0, 1.0) // Ensure that we do not exceed outside 0.0 and 1.0 //TODO do we need this? Do we want to help the user in single step operations or be accurate in multistep operations
        });
        Ok(())
    }
    
    pub fn change_contrast(&mut self, contrast_factor: f32) -> Result<(), FeagiDataError> { // TODO This algorithm is likely wrong given linear and gamma color spaces!
        if contrast_factor < -1.0 || contrast_factor > 1.0 {
            return Err(FeagiDataError::BadParameters("The contrast factor must be between -1.0 and 1.0!".into()).into());
        }
        // Algo sourced from https://ie.nitk.ac.in/blog/2020/01/19/algorithms-for-adjusting-brightness-and-contrast-of-an-image/
        const CORRECTION_FACTOR: f32 = 1.015686; //  259 / 255
        self.pixels.mapv_inplace(|v| {
            let factor: f32 = (CORRECTION_FACTOR * (contrast_factor + 1.0)) / (CORRECTION_FACTOR - contrast_factor);
            let pixel_val: f32 = (factor * (v - 0.5)) + 0.5;
            pixel_val.clamp(0.0, 1.0)
        });
        Ok(())
    }


    //endregion

    //region Out-Place
    
    pub fn resize_nearest_neighbor(&mut self, target_width_height: ImageXYResolution) -> Result<(), FeagiDataError> {
        let source_resolution: ImageXYResolution = self.get_xy_resolution();
        let source_resolution: (u32, u32) = (source_resolution.height as u32, source_resolution.width as u32);
        let source_resolution_f: (f32, f32) = (source_resolution.0 as f32, source_resolution.1 as f32); // Y X order
        let number_color_channels: usize = self.get_color_channel_count();

        let mut sized_array: Array3<f32> = Array3::zeros((target_width_height.height, target_width_height.width, number_color_channels));
        let target_width_height_f: (f32, f32) = (target_width_height.width as f32, target_width_height.height as f32);
        for ((y, x, c), color_val) in sized_array.indexed_iter_mut() {
            let nearest_neighbor_coordinate_y: usize = (((y as f32) / source_resolution_f.0) * target_width_height_f.1).floor() as usize;
            let nearest_neighbor_coordinate_x: usize = (((x as f32) / source_resolution_f.1) * target_width_height_f.0).floor() as usize;
            let nearest_neighbor_channel_value: f32 = self.pixels[(nearest_neighbor_coordinate_x, nearest_neighbor_coordinate_y, c)];
            *color_val = nearest_neighbor_channel_value;
        };
        self.pixels = sized_array;

        Ok(())
    }

    //endregion


    //endregion

    // region Outputting Neurons

    pub fn write_as_neuron_xyzp_data(&self, write_target: &mut CorticalMappedXYZPNeuronData, target_id: CorticalID, x_channel_offset: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        const EPSILON: f32 = 0.0001; // avoid writing near zero vals
        const MIN_PIXEL_VAL: f32 = 0.0;
        const MAX_PIXEL_VAL: f32 = 1.0;

        let y_flip_distance: u32 = self.get_xy_resolution().height as u32;
        let x_offset: u32 = *x_channel_offset * self.get_xy_resolution().width as u32;
        let mapped_neuron_data = write_target.ensure_clear_and_borrow_mut(&target_id, self.get_number_elements());

        mapped_neuron_data.update_vectors_from_external(|x_vec, y_vec, c_vec, p_vec| {
            for ((y, x, c), color_val) in self.pixels.indexed_iter() { // going from row major to cartesian
                if color_val.abs() > EPSILON {
                    x_vec.push(x as u32 + x_offset);
                    y_vec.push( y_flip_distance - y as u32);  // flip y
                    c_vec.push(c as u32);
                    p_vec.push((*color_val).clamp(MIN_PIXEL_VAL, MAX_PIXEL_VAL));
                }
            };
            Ok(())
        })


    }

    // endregion
    
    // region Internal Functions

    /// Converts an array from any memory layout to row-major (HeightsWidthsChannels) format.
    ///
    /// This internal function handles the conversion of arrays between different memory
    /// layouts, ensuring they are stored in the standard row-major format used internally.
    ///
    /// # Arguments
    ///
    /// * `input` - The input array to convert
    /// * `source_memory_order` - The current memory layout of the input array
    ///
    /// # Returns
    ///
    /// The converted array in row-major format.
    fn change_memory_order_to_row_major(input: Array3<f32>, source_memory_order: &MemoryOrderLayout) -> Array3<f32> {
        match source_memory_order {
            MemoryOrderLayout::HeightsWidthsChannels => input, // Nothing needed, we store in this format anyway
            MemoryOrderLayout::ChannelsHeightsWidths => input.permuted_axes([2, 0, 1]),
            MemoryOrderLayout::WidthsHeightsChannels => input.permuted_axes([1, 0, 2]),
            MemoryOrderLayout::HeightsChannelsWidths => input.permuted_axes([0, 2, 1]),
            MemoryOrderLayout::ChannelsWidthsHeights => input.permuted_axes([2, 1, 0]),
            MemoryOrderLayout::WidthsChannelsHeights => input.permuted_axes([1, 2, 0]),
        }
    }

    /// Converts an array from row-major format to any other memory layout.
    ///
    /// This internal function handles the conversion of arrays from the internal
    /// row-major format to any other specified memory layout.
    ///
    /// # Arguments
    ///
    /// * `input` - The input array in row-major format
    /// * `target_memory_order` - The desired memory layout for the output array
    ///
    /// # Returns
    ///
    /// The converted array in the target memory layout.
    fn change_memory_order_from_row_major(input: Array3<f32>, target_memory_order: &MemoryOrderLayout) -> Array3<f32> {
        match target_memory_order {
            MemoryOrderLayout::HeightsWidthsChannels => input, // Nothing needed, we store in this format anyway
            MemoryOrderLayout::ChannelsHeightsWidths => input.permuted_axes([1, 2, 0]),
            MemoryOrderLayout::WidthsHeightsChannels => input.permuted_axes([1, 0, 2]),
            MemoryOrderLayout::HeightsChannelsWidths => input.permuted_axes([0, 2, 1]),
            MemoryOrderLayout::ChannelsWidthsHeights => input.permuted_axes([2, 1, 0]),
            MemoryOrderLayout::WidthsChannelsHeights => input.permuted_axes([2, 0, 1]),
        }
    }

    /// Validates that an incoming ImageFrame has compatible color properties.
    ///
    /// This internal function checks that the incoming frame has the same color space
    /// and channel format as this frame, which is required for in-place operations.
    ///
    /// # Arguments
    ///
    /// * `incoming` - The ImageFrame to validate for compatibility
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// - Ok(()) if the frames are compatible
    /// - Err(DataProcessingError) if the color properties don't match
    fn err_if_incoming_image_frame_is_color_incompatible(&self, incoming: &ImageFrame) -> Result<(), FeagiDataError> {
        if self.color_space != incoming.color_space {
            return Err(FeagiDataError::BadParameters("Incoming source array does not have matching color space!".into()).into())
        }
        if self.channel_layout == incoming.channel_layout {
            return Err(FeagiDataError::BadParameters("Incoming source array does not have matching color channel count!!".into()).into())
        }
        Ok(())
    }

    // endregion
    
}


impl std::fmt::Display for ImageFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ImageFrame({})", self.get_image_frame_properties())
    }
}