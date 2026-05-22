



/// Implemented by all FEAGI Errors
pub trait FeagiErrorBase: Sized {
    // TODO create paths for converting to string, debug, or embedded debug, all properly feature gated
}




// TODO some sort of macro + struct + trait setup for the individual keys to standardize the text outputs of the errors

// TODO macro that automatically implements the into for a child error into its parent