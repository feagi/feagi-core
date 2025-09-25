
/// Mainly used in macros, but denotes the type of encoder / decoder to be used
#[allow(non_camel_case_types)]
pub enum CoderTypes {
    Percentage_Instant_Linear,
    Percentage_Instant_FractionalExponential,
    Percentage_Incremental_Linear,
    Percentage_Incremental_FractionalExponential,

    Percentage2D_Instant_Linear,
    Percentage2D_Instant_FractionalExponential,
    Percentage2D_Incremental_Linear,
    Percentage2D_Incremental_FractionalExponential,

    Percentage3D_Instant_Linear,
    Percentage3D_Instant_FractionalExponential,
    Percentage3D_Incremental_Linear,
    Percentage3D_Incremental_FractionalExponential,

    Percentage4D_Instant_Linear,
    Percentage4D_Instant_FractionalExponential,
    Percentage4D_Incremental_Linear,
    Percentage4D_Incremental_FractionalExponential,

    SignedPercentage_Instant_Linear,
    SignedPercentage_Instant_FractionalExponential,
    SignedPercentage_Incremental_Linear,
    SignedPercentage_Incremental_FractionalExponential,

    SignedPercentage2D_Instant_Linear,
    SignedPercentage2D_Instant_FractionalExponential,
    SignedPercentage2D_Incremental_Linear,
    SignedPercentage2D_Incremental_FractionalExponential,

    SignedPercentage3D_Instant_Linear,
    SignedPercentage3D_Instant_FractionalExponential,
    SignedPercentage3D_Incremental_Linear,
    SignedPercentage3D_Incremental_FractionalExponential,

    SignedPercentage4D_Instant_Linear,
    SignedPercentage4D_Instant_FractionalExponential,
    SignedPercentage4D_Incremental_Linear,
    SignedPercentage4D_Incremental_FractionalExponential,

    MiscData_Instant,
    MiscData_Incremental,

    ImageFrame_Instant,
    ImageFrame_Incremental,

    SegmentedImageFrame_Instant,
    SegmentedImageFrame_Incremental,
}