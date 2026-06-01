use crate::tag_device::PDITagGenericDevice;

/// Denotes that the element data lives on some device. As such, anything inheriting only this
/// trait directly cannot have any data access methods of any sort. It needs to be extended first.
/// Note that technically Elements can "group" other elements as well, or even multiple.
/// This is essentially just a tag
pub trait PDIElement: PDITagGenericDevice + Sized {
    // Extend this with the data type being represented that, and then extend that with the
    // PDITag representing where it can live
}


/// Shared trait among all Collections. While only those that extend PDITagCPU
/// can have their internal data accessible and workable from standard Rust CPU systems, it is
/// possible for structs implementing other devices to still have "Metadata" that allows the CPU
/// to at least have some context of what is being held. Further more, it is these that parallel
/// operations are ran upon by executors
pub trait PDICollection: PDITagGenericDevice
{ 
    // Extend this with the PDIElement(s) being held, and possibly extend that with the
    // implementations of the different devices
}

/// Implementations of this act upon one (or more) PDICollections to efficiently in parallel
/// perform some operation
pub trait PDIExecutor {
    // Extend this with the data type being represented that, and then extend that with the
    // PDITag representing where it can live
}