

/// Represents a struct with pub fields corresponding to a tuple of `FeagiMessagePath` that represent 
/// a messages data path (and points to the data structure type) and a given data type to be coupled
/// to it (depending on the usecase)
pub trait FeagiCategoryGroup<T>  {
    // Struct should be `pub message_name: (FeagiMessagePath, T)`
}

/// Represents a struct with pub fields corresponding to a tuple of `FeagiCategoryGroup` that represent 
/// a category of message paths and a given data type to be coupled
/// to it (depending on the usecase)
pub trait FeagiGroupedCategories<T>  {
    // Struct should be `pub category_name: (FeagiMessagePath<X>, T)`
}