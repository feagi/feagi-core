
/// Mainly used in macros, but denotes the type of encoder / decoder to be used
#[allow(non_camel_case_types)]
pub enum CoderTypes {
    Percentage_Absolute_Linear,
    Percentage_Absolute_Fractional,
    Percentage_Incremental_Linear,
    Percentage_Incremental_Fractional,

    Percentage2D_Absolute_Linear,
    Percentage2D_Absolute_Fractional,
    Percentage2D_Incremental_Linear,
    Percentage2D_Incremental_Fractional,

    Percentage3D_Absolute_Linear,
    Percentage3D_Absolute_Fractional,
    Percentage3D_Incremental_Linear,
    Percentage3D_Incremental_Fractional,

    Percentage4D_Absolute_Linear,
    Percentage4D_Absolute_Fractional,
    Percentage4D_Incremental_Linear,
    Percentage4D_Incremental_Fractional,

    SignedPercentage_Absolute_Linear,
    SignedPercentage_Absolute_Fractional,
    SignedPercentage_Incremental_Linear,
    SignedPercentage_Incremental_Fractional,

    SignedPercentage2D_Absolute_Linear,
    SignedPercentage2D_Absolute_Fractional,
    SignedPercentage2D_Incremental_Linear,
    SignedPercentage2D_Incremental_Fractional,

    SignedPercentage3D_Absolute_Linear,
    SignedPercentage3D_Absolute_Fractional,
    SignedPercentage3D_Incremental_Linear,
    SignedPercentage3D_Incremental_Fractional,

    SignedPercentage4D_Absolute_Linear,
    SignedPercentage4D_Absolute_Fractional,
    SignedPercentage4D_Incremental_Linear,
    SignedPercentage4D_Incremental_Fractional,

    MiscData_Absolute,
    MiscData_Incremental,

    ImageFrame_Absolute,
    ImageFrame_Incremental,

    SegmentedImageFrame_Absolute,
    SegmentedImageFrame_Incremental,
}