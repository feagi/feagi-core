
/// Mainly used in macros, but denotes the type of encoder / decoder to be used
#[allow(non_camel_case_types)]
pub enum CoderTypes {
    Percentage_Instant_Linear,
    Percentage_Instant_Fractional,
    Percentage_Incremental_Linear,
    Percentage_Incremental_Fractional,

    Percentage2D_Instant_Linear,
    Percentage2D_Instant_Fractional,
    Percentage2D_Incremental_Linear,
    Percentage2D_Incremental_Fractional,

    Percentage3D_Instant_Linear,
    Percentage3D_Instant_Fractional,
    Percentage3D_Incremental_Linear,
    Percentage3D_Incremental_Fractional,

    Percentage4D_Instant_Linear,
    Percentage4D_Instant_Fractional,
    Percentage4D_Incremental_Linear,
    Percentage4D_Incremental_Fractional,

    SignedPercentage_Instant_Linear,
    SignedPercentage_Instant_Fractional,
    SignedPercentage_Incremental_Linear,
    SignedPercentage_Incremental_Fractional,

    SignedPercentage2D_Instant_Linear,
    SignedPercentage2D_Instant_Fractional,
    SignedPercentage2D_Incremental_Linear,
    SignedPercentage2D_Incremental_Fractional,

    SignedPercentage3D_Instant_Linear,
    SignedPercentage3D_Instant_Fractional,
    SignedPercentage3D_Incremental_Linear,
    SignedPercentage3D_Incremental_Fractional,

    SignedPercentage4D_Instant_Linear,
    SignedPercentage4D_Instant_Fractional,
    SignedPercentage4D_Incremental_Linear,
    SignedPercentage4D_Incremental_Fractional,

    MiscData_Instant,
    MiscData_Incremental,

    ImageFrame_Instant,
    ImageFrame_Incremental,

    SegmentedImageFrame_Instant,
    SegmentedImageFrame_Incremental,
}