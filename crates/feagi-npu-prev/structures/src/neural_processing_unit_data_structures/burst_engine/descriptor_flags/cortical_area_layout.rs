


/// Represents what type of cortical layout is being used in a cortical area, within 3 bits
/// (limiting to only 8 options)
#[repr(u8)]
#[derive(Copy, Clone, Default)]
pub enum CorticalAreaLayoutType {
    #[default]
    Dimensional = 0,
    Memory = 1,
}



