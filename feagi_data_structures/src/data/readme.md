# IO Data
These types describe the numerical data that can be flown in and out of FEAGI that can be easily used.

## Wrapping
Due to Rust type safety, data is wrapped to allow easy use in dynamic contexts.

### IO Type Data
An enum that is used to wrap data into a single form to ensure type safety in Rust.

### IO Type Variant
An enum that is used to easily describe the type of data. Used by functions to enforce accepted IOTypeData variants.

## Images

### Image Frame
Represents a single image, typically as a frame from a video camera feed. Uncompressed.

### Segmented Image Frame
Represents 9 Image Frames in a 3x3 grid, where the center image is often higher resolution than the rest. This is intended to mimic how animals have a central vision of focus, in higher resolution, while the peripheral region is lower resolution.

### Descriptors
These simple structs / enums are used in various image processing functions and to describe the data in the images.

#### Segmented Frame Target Resolutions
Describes the X Y resolution of each image frame in a segmented image frame.

#### Segmented Frame Center Properties
Describes the "focus" of the center image from in a segmented image frame, by specifying a normalized 0-1 center point on the x-y, and the size of the center in relation to the rest in the x-y direction with a normalized 0-1 value per axis. Used when taking in an input image frame and processing it into a segmented image frame.

#### Memory Order Layout
Enum that describes the ordering of the data axis wise being inputted. Relevant when directly importing raw data.

#### Channel Format
Enum Describing the number of color channels in an image frame, ranging from 1 (R / Grayscale) to 4 (RGBA)

#### Color Space
Enum describing if the color pixel data is in linear or gamma space.

#### Corner Points
Specifies the corners of each output image frame in a generate segmented image frame from an input image frame. Used if user wishes to be explicit in defining the boundaries of a segmented image frame, otherwise is used only internally.

#### Frame Processing Parameters
Specifies the all of processing you wish to do to an image frame. Allows for convenient storing of the configuration and can be used instead of running multiple processing steps independently, where this system can attempt to combine certain operations to be more efficient.

## Ranged Floats
Specifies floats within certain ranges

### Linear Bounded F32
A F32 bounded between an upper and lower bound. Useful in the cases of motor outputs where you want to enforce a data range.

### Linear Normalized F32
A F32 bounded specifically between -1 and 1. Useful as technically all input / output cortical areas with float type channels, can only process data in this range directly.

## JSON Structure
Essentially only used for command / control purposes to FEAGI within the FEAGI Byte Structure wrapper. Not for user use.