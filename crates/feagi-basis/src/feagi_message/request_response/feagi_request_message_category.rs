
/// The enum type that messages of a category can convert into
pub trait FeagiRequestMessageCategoryEnum {}


pub trait FeagiRequestMessageCategory {
    type CategoryEnum: FeagiRequestMessageCategoryEnum;
}



/// Represents a struct with pub fields corresponding to a tuple of `FeagiCategoryGroup` that represent
/// a category of message paths and a given data type to be coupled
/// to it (depending on the usecase)
pub trait FeagiGroupedRequestMessageCategorizedMappings<T> {
    // Struct should be `pub category_name: (FeagiRequestMessageCategorizedMappings<X>, T)`
}
