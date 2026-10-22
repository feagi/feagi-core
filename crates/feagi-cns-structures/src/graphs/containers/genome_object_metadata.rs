
/// Stores data that all genome objects have. Bears no computational consequence.
pub struct GenomeObjectMetadata {
    pub name: String,
    pub description: String,
    pub position_3d: (),
    pub position_2d: (),
}

// TODO sizing limits?