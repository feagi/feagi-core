use ndarray::{s, ArrayView3};
use crate::FeagiDataError;
use crate::data::image_descriptors::{ColorChannelLayout, ColorSpace, CornerPoints, ImageFrameProperties, ImageXYResolution};
use crate::data::ImageFrame;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ImageFrameProcessor {
    /// Properties that the input image must match (resolution, color space, channel layout)
    input_image_properties: ImageFrameProperties,
    /// Optional cropping region defined by corner points
    cropping_from: Option<CornerPoints>, 
    /// Optional target resolution for resizing operation
    final_resize_xy_to: Option<ImageXYResolution>,
    /// Optional target color space for conversion
    convert_color_space_to: Option<ColorSpace>,
    /// Optional brightness multiplier factor
    multiply_brightness_by: Option<f32>,
    /// Optional contrast adjustment factor
    change_contrast_by: Option<f32>,
    /// Whether to convert the image to grayscale (only allowed on RGB/RGBA images)
    convert_to_grayscale: bool,
}

impl std::fmt::Display for ImageFrameProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let initial = format!("Expecting {}.", self.input_image_properties);
        let mut steps: String = match (self.cropping_from, self.final_resize_xy_to) {
            (None, None) => format!("Keeping input size of {} (no cropping from or resizing to)", self.input_image_properties.get_image_resolution()),
            (Some(cropping_from), None) => format!("Cropping from xy points <{}, {}> to <{}, {}> without resizing after,",
                                                   cropping_from.lower_left_row_major().1, cropping_from.lower_left_row_major().0, cropping_from.upper_right_row_major().1, cropping_from.upper_right_row_major().0),
            (None, Some(final_resize_xy_to)) => format!("resizing to resolution {} without any cropping,", final_resize_xy_to),
            (Some(cropping_from), Some(final_resize_xy_to)) => format!("Cropping from xy points <{}, {}> to <{}, {}> then resizing to resolution {},",
                                                                       cropping_from.lower_left_row_major().1, cropping_from.lower_left_row_major().0, cropping_from.upper_right_row_major().1, cropping_from.upper_right_row_major().0, final_resize_xy_to),
        };
        steps += &*(match self.convert_color_space_to {
            None => String::new(),
            Some(change_colorspace_to) => format!("Convert Colorspace to {}", change_colorspace_to.to_string()),
        });
        steps += &*(match self.multiply_brightness_by {
            None => String::new(),
            Some(multiply_brightness_by) => format!("Multiply brightness by {}", multiply_brightness_by),
        });
        steps += &*(match self.change_contrast_by {
            None => String::new(),
            Some(change_contrast_by) => format!("Change contrast by {}", change_contrast_by),
        });
        steps += &*(match self.convert_to_grayscale {
            false => String::new(),
            true => "Convert to grayscale".to_string(),
        });
        write!(f, "ImageFrameCleanupDefinition({} {})", initial, steps)
    }
}

impl ImageFrameProcessor {
    
    pub fn new(input_image_properties: ImageFrameProperties) -> ImageFrameProcessor {
        ImageFrameProcessor {
            input_image_properties,
            cropping_from: None,
            final_resize_xy_to: None,
            multiply_brightness_by: None,
            change_contrast_by: None,
            convert_color_space_to: None,
            convert_to_grayscale: false,
        }
    }

    pub fn new_from_input_output_properties(input: &ImageFrameProperties, output: &ImageFrameProperties) -> Result<Self, FeagiDataError> {
        let mut definition = ImageFrameProcessor::new(input.clone());
        if output.get_color_channel_layout() != input.get_color_channel_layout() {
            if output.get_color_channel_layout() == ColorChannelLayout::GrayScale && input.get_color_channel_layout() == ColorChannelLayout::RGB {
                // supported
                definition.convert_to_grayscale = true;
            }
            // unsupported
            return Err(FeagiDataError::BadParameters("Given Color Conversion not possible!".into()).into())
        }
        if output.get_image_resolution() != input.get_image_resolution() {
            definition.set_resizing_to(output.get_image_resolution())?;
        }
        if output.get_color_space() != output.get_color_space() {
            definition.set_color_space_to(&output.get_color_space())?;
        }
        Ok(definition)
    }


    pub fn get_input_image_properties(&self) -> &ImageFrameProperties { &self.input_image_properties }
    

    pub fn get_output_image_properties(&self) -> ImageFrameProperties {
        let resolution = match (self.cropping_from, self.final_resize_xy_to) {
            (None, None) => self.input_image_properties.get_image_resolution(),
            (Some(cropping_from), None) => cropping_from.enclosed_area_width_height(),
            (None, Some(final_resize_xy_to)) => final_resize_xy_to,
            (Some(_), Some(final_resize_xy_to)) => final_resize_xy_to,
        };
        let color_space = match self.convert_color_space_to {
            None => self.input_image_properties.get_color_space(),
            Some(color_space_to) => color_space_to,
        };
        let color_channel_layout = match self.convert_to_grayscale {
            false => self.input_image_properties.get_color_channel_layout(),
            true => ColorChannelLayout::GrayScale,
        };
        ImageFrameProperties::new(resolution, color_space, color_channel_layout).unwrap()
    }
    

    pub fn verify_input_image_allowed(&self, verifying_image: &ImageFrame) -> Result<(), FeagiDataError> {
        self.input_image_properties.verify_image_frame_matches_properties(verifying_image)
    }
    

    // TODO 2 / 4 channel pipelines!
    // Due to image segmentor, I would argue the most common route is crop + resize + grayscale
    pub fn process_image(&self, source: &ImageFrame, destination: &mut ImageFrame) -> Result<(), FeagiDataError> {
        match self {
            // Do literally nothing, just copy the data
            ImageFrameProcessor {
                input_image_properties,
                cropping_from: None,
                final_resize_xy_to: None,
                convert_color_space_to: None,
                multiply_brightness_by: None,
                change_contrast_by: None,
                convert_to_grayscale: false
            } => {
                *destination = source.clone();
                Ok(())
            }

            // Only cropping
            ImageFrameProcessor {
                input_image_properties,
                cropping_from: Some(cropping_from),
                final_resize_xy_to: None,
                convert_color_space_to: None,
                multiply_brightness_by: None,
                change_contrast_by:None,
                convert_to_grayscale: false
            } => {
                crop(source, destination, cropping_from, self.get_output_channel_count())
            }

            // Only resizing
            ImageFrameProcessor {
                input_image_properties,
                cropping_from: None,
                final_resize_xy_to: Some(final_resize_xy_to),
                convert_color_space_to: None,
                multiply_brightness_by: None,
                change_contrast_by:None,
                convert_to_grayscale: false
            } => {
                resize(source, destination, final_resize_xy_to)
            }

            // Only grayscaling
            ImageFrameProcessor {
                input_image_properties,
                cropping_from: None,
                final_resize_xy_to: None,
                convert_color_space_to: None,
                multiply_brightness_by: None,
                change_contrast_by:None,
                convert_to_grayscale: true
            } => {
                to_grayscale(source, destination, self.input_image_properties.get_color_space())
            }

            // Cropping, Resizing
            ImageFrameProcessor {
                input_image_properties,
                cropping_from: Some(cropping_from),
                final_resize_xy_to: Some(final_resize_xy_to),
                convert_color_space_to: None,
                multiply_brightness_by: None,
                change_contrast_by:None,
                convert_to_grayscale: false
            } => {
                crop_and_resize(source, destination, cropping_from, final_resize_xy_to)
            }

            // Cropping, Resizing, Grayscaling (the most common with segmentation vision)
            ImageFrameProcessor {
                input_image_properties,
                cropping_from: Some(cropping_from),
                final_resize_xy_to: Some(final_resize_xy_to),
                convert_color_space_to: None,
                multiply_brightness_by: None,
                change_contrast_by:None,
                convert_to_grayscale: true
            } => {
                crop_and_resize_and_grayscale(source, destination, cropping_from, final_resize_xy_to, self.input_image_properties.get_color_space())
            }

            // If no fast path, use this slower universal one
            _ => {
                // This function is much slower, There may be some optimization work possible, but ensure the most common step combinations have an accelerated path
                let is_cropping_is_resizing = (self.cropping_from, self.final_resize_xy_to);

                let mut processing = source.clone();
                match is_cropping_is_resizing {
                    (None, None) => {
                        // don't do anything
                    }
                    (Some(cropping_from), None) => {
                        crop(source, &mut processing, &cropping_from, self.get_output_channel_count())?;
                    }
                    (None, Some(final_resize_xy_to)) => {
                        resize(source, &mut processing, &final_resize_xy_to)?;
                    }
                    (Some(cropping_from), Some(final_resize_xy_to)) => {
                        crop_and_resize(source, &mut processing, &cropping_from, &final_resize_xy_to)?;
                    }
                };

                match self.convert_color_space_to {
                    None => {
                        // Do Nothing
                    }
                    Some(color_space) => {
                        return Err(FeagiDataError::NotImplemented)
                    }
                }

                match self.multiply_brightness_by {
                    None => {
                        // Do Nothing
                    }
                    Some(brightness_multiplier) => {
                        processing.change_brightness(brightness_multiplier)?;
                    }
                }

                match self.change_contrast_by {
                    None => {
                        // Do Nothing
                    }
                    Some(contrast_multiplier) => {
                        processing.change_contrast(contrast_multiplier)?;
                    }
                }

                if self.convert_to_grayscale {
                    return Err(FeagiDataError::NotImplemented)
                }

                *destination = processing;
                Ok(())
            }

        }


    }

    //region set settings
    // TODO safety bound checks!
    
    
    pub fn set_cropping_from(&mut self, lower_left_xy_point_inclusive: (usize, usize), upper_right_xy_point_exclusive: (usize, usize)) -> Result<&mut Self, FeagiDataError> {
        let corner_points = CornerPoints::new_from_cartesian(lower_left_xy_point_inclusive, upper_right_xy_point_exclusive, self.input_image_properties.get_image_resolution())?;
        self.cropping_from = Some(corner_points);
        Ok(self)
    }


    pub fn set_resizing_to(&mut self, new_xy_resolution: ImageXYResolution) -> Result<&mut Self, FeagiDataError> {
        self.final_resize_xy_to = Some(new_xy_resolution);
        Ok(self)
    }


    pub fn set_brightness_multiplier(&mut self, brightness_multiplier: f32) -> Result<&mut Self, FeagiDataError> {
        if brightness_multiplier == 1.0 {
            self.multiply_brightness_by = None;
        }
        else {
            self.multiply_brightness_by = Some(brightness_multiplier);
        }
        Ok(self)
    }


    pub fn set_contrast_change(&mut self, contrast_change: f32) -> Result<&mut Self, FeagiDataError> {
        if contrast_change == 1.0 {
            self.change_contrast_by = None;
        }
        else {
            self.change_contrast_by = Some(contrast_change);
        }
        Ok(self)
    }


    pub fn set_color_space_to(&mut self, color_space: &ColorSpace) -> Result<&mut Self, FeagiDataError> {
        if color_space == &self.input_image_properties.get_color_space() {
            self.convert_color_space_to = None;
        }
        else {
            self.convert_color_space_to = Some(*color_space);
        }
        Ok(self)
    }


    pub fn set_conversion_to_grayscale(&mut self, convert_to_grayscale: bool) -> Result<&mut Self, FeagiDataError> {
        if self.input_image_properties.get_color_channel_layout() == ColorChannelLayout::RG {
            return Err(FeagiDataError::NotImplemented)
        }
        self.convert_to_grayscale = convert_to_grayscale;
        Ok(self)
    }

    //region clear settings
    
    pub fn clear_all_transformations(&mut self) -> &Self {
        self.cropping_from = None;
        self.final_resize_xy_to = None;
        self.convert_color_space_to = None;
        self.multiply_brightness_by = None;
        self.change_contrast_by = None;
        self.convert_to_grayscale = false;
        self
    }
    
    pub fn clear_cropping(&mut self) -> &Self {
        self.cropping_from = None;
        self
    }
    
    pub fn clear_resizing(&mut self) -> &Self {
        self.final_resize_xy_to = None;
        self
    }
    
    pub fn clear_brightness_adjustment(&mut self) -> &Self {
        self.multiply_brightness_by = None;
        self
    }
    
    pub fn clear_contrast_adjustment(&mut self) -> &Self {
        self.change_contrast_by = None;
        self
    }
    
    pub fn clear_color_space_conversion(&mut self) -> &Self {
        self.convert_color_space_to = None;
        self
    }
    
    pub fn clear_grayscale_conversion(&mut self) -> &Self {
        self.convert_to_grayscale = false;
        self
    }

    //endregion

    //endregion
    
    //region helpers

    fn get_output_channel_count(&self) -> usize {
        if self.convert_to_grayscale {
            return 1;
        }
        self.input_image_properties.get_color_channel_layout().into()
    }
    

    //endregion
    

    
}

//region source destination processing

fn crop(source: &ImageFrame, destination: &mut ImageFrame, crop_from: &CornerPoints, number_output_color_channels: usize) -> Result<(), FeagiDataError> {
    let mut destination_data = destination.get_internal_data_mut();
    let sliced_array_view: ArrayView3<f32> = source.get_internal_data().slice(
        s![crop_from.lower_left_row_major().0 .. crop_from.upper_right_row_major().0,
                crop_from.lower_left_row_major().1 .. crop_from.upper_right_row_major().1 , 0..number_output_color_channels]
    );
    destination_data = &mut sliced_array_view.into_owned();
    Ok(())
}

fn resize(source: &ImageFrame, destination: &mut ImageFrame, resize_xy_to: &ImageXYResolution) -> Result<(), FeagiDataError> {
    // Uses Nearest Neighbor. Not pretty but fast
    let source_resolution = source.get_xy_resolution();
    let source_resolution: (u32, u32) = (source_resolution.height as u32, source_resolution.width as u32);
    let dest_resolution_f: (f32, f32) = (resize_xy_to.width as f32, resize_xy_to.height as f32);
    let source_resolution_f: (f32, f32) = (source_resolution.0 as f32, source_resolution.1 as f32);

    let source_data = source.get_internal_data();
    let destination_data = destination.get_internal_data_mut();

    for ((y,x,c), color_val) in destination_data.indexed_iter_mut() {
        let nearest_neighbor_coordinate_y: usize = (((y as f32) / dest_resolution_f.1) * source_resolution_f.1).floor() as usize;
        let nearest_neighbor_coordinate_x: usize = (((x as f32) / dest_resolution_f.0) * source_resolution_f.0).floor() as usize;
        let nearest_neighbor_channel_value: f32 = source_data[(
            nearest_neighbor_coordinate_x,
            nearest_neighbor_coordinate_y,
            c)];
        *color_val = nearest_neighbor_channel_value;
    };
    Ok(())
}

fn to_grayscale(source: &ImageFrame, destination: &mut ImageFrame, output_color_space: ColorSpace) -> Result<(), FeagiDataError> {
    // NOTE: destination should be grayscale and source should be RGB or RGBA
    let source_data = source.get_internal_data();
    let destination_data = destination.get_internal_data_mut();
    let (r_scale, g_scale, b_scale) = match output_color_space {
        ColorSpace::Linear => {(0.2126f32, 0.7152f32, 0.072f32)} // Using formula from https://stackoverflow.com/questions/17615963/standard-rgb-to-grayscale-conversion
        ColorSpace::Gamma => {(0.299f32, 0.587f32, 0.114f32)}  // https://www.youtube.com/watch?v=uKeKuaJ4nlw (I forget)
    };
    // TODO look into premultiplied alpha handling!

    for ((y,x,c), color_val) in destination_data.indexed_iter_mut() {
        // TODO this is bad, we shouldnt be iterating over color channel and matching like this. Major target for optimization!
        if c != 0 { continue; }
        *color_val = r_scale * source_data[(y, x, 0)] + b_scale * source_data[(y, x, 1)] + g_scale * source_data[(y, x, 2)];
    }
    Ok(())
    

    
}

fn crop_and_resize(source: &ImageFrame, destination: &mut ImageFrame, crop_from: &CornerPoints, resize_xy_to: &ImageXYResolution) -> Result<(), FeagiDataError> {

    let crop_resolution: ImageXYResolution = crop_from.enclosed_area_width_height();
    let resolution_f: (f32, f32) = (resize_xy_to.width as f32, resize_xy_to.height as f32);
    let crop_resolution_f: (f32, f32) = (crop_resolution.width as f32, crop_resolution.height as f32);

    let dist_factor_yx: (f32, f32) = (
        crop_resolution_f.1 / resolution_f.1,
        crop_resolution_f.0 / resolution_f.0);

    let upper_left_corner_offset_yx: (usize, usize) = (
        crop_from.upper_left_row_major().0,
        crop_from.upper_left_row_major().1,
    );

    let source_data = source.get_internal_data();
    let destination_data = destination.get_internal_data_mut();

    for ((y,x,c), color_val) in destination_data.indexed_iter_mut() {
        let nearest_neighbor_coordinate_from_source_y: usize = ((y as f32) * dist_factor_yx.0).floor() as usize;
        let nearest_neighbor_coordinate_from_source_x: usize = ((x as f32) * dist_factor_yx.1).floor() as usize;
        let nnc_y_with_offset = nearest_neighbor_coordinate_from_source_y + upper_left_corner_offset_yx.0;
        let nnc_x_with_offset = nearest_neighbor_coordinate_from_source_x + upper_left_corner_offset_yx.1;

        let nearest_neighbor_channel_value: f32 = source_data[(
            nnc_y_with_offset,
            nnc_x_with_offset,
            c)];
        *color_val = nearest_neighbor_channel_value;
    };
    Ok(())
}

fn crop_and_resize_and_grayscale(source: &ImageFrame, destination: &mut ImageFrame, crop_from: &CornerPoints, resize_xy_to: &ImageXYResolution, output_color_space: ColorSpace) -> Result<(), FeagiDataError> {

    let crop_resolution: ImageXYResolution = crop_from.enclosed_area_width_height();
    let resolution_f: (f32, f32) = (resize_xy_to.width as f32, resize_xy_to.height as f32);
    let crop_resolution_f: (f32, f32) = (crop_resolution.width as f32, crop_resolution.height as f32);

    let dist_factor_yx: (f32, f32) = (
        crop_resolution_f.1 / resolution_f.1,
        crop_resolution_f.0 / resolution_f.0);

    let upper_left_corner_offset_yx: (usize, usize) = (
        crop_from.upper_left_row_major().0,
        crop_from.upper_left_row_major().1,
    );

    let source_data = source.get_internal_data();
    let destination_data = destination.get_internal_data_mut();
    let (r_scale, g_scale, b_scale) = match output_color_space {
        ColorSpace::Linear => {(0.2126f32, 0.7152f32, 0.072f32)} // Using formula from https://stackoverflow.com/questions/17615963/standard-rgb-to-grayscale-conversion
        ColorSpace::Gamma => {(0.299f32, 0.587f32, 0.114f32)}
    };
    // TODO look into premultiplied alpha handling!

    for ((y,x,c), color_val) in destination_data.indexed_iter_mut() {
        // TODO this is bad, we shouldnt be iterating over color channel and matching like this. Major target for optimization!
        if c != 0 { continue; }
        let nearest_neighbor_coordinate_from_source_y: usize = ((y as f32) * dist_factor_yx.0).floor() as usize;
        let nearest_neighbor_coordinate_from_source_x: usize = ((x as f32) * dist_factor_yx.1).floor() as usize;
        let nnc_y_with_offset = nearest_neighbor_coordinate_from_source_y + upper_left_corner_offset_yx.0;
        let nnc_x_with_offset = nearest_neighbor_coordinate_from_source_x + upper_left_corner_offset_yx.1;
        let nearest_neighbor_channel_r: f32 = source_data[(
            nnc_y_with_offset,
            nnc_x_with_offset,
            0)];
        let nearest_neighbor_channel_g: f32 = source_data[(
            nnc_y_with_offset,
            nnc_x_with_offset,
            1)];
        let nearest_neighbor_channel_b: f32 = source_data[(
            nnc_y_with_offset,
            nnc_x_with_offset,
            2)];
        
        *color_val = r_scale * nearest_neighbor_channel_r + b_scale * nearest_neighbor_channel_g + g_scale * nearest_neighbor_channel_b;
    }
    Ok(())
    
}
//endregion
