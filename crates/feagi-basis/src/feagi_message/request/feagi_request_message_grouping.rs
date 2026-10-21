use crate::feagi_message::request::feagi_request_descriptor::FeagiRequestMessageDescriptor;
use crate::feagi_message::request::feagi_request_message_category::{FeagiCategoryRequester, FeagiRequestCategory};
use crate::parallelism::feagi_channel::request_response_channel::channel_backend::FeagiRequestResponseChannelBackend;

//region Category

/// Nests the message descriptors of a category, as `NMDes<Head, NMDes<..., NNull>>`
pub struct NMDes<Head, Tail>
where
    Head: FeagiRequestMessageDescriptor,
    // Tail can be either another NMDes or a NNull (tail)
    {
    pub head: Head,
    pub tail: Tail
}

/// Tail of the `NMDes` and `NCat`
pub struct NNull;

//endregion

//region Category Group

/// A group of categories, passed around as one so a worker can serve or make requests of all
/// of them. Purely a grouping, each category is independent with its own channel and request /
/// response enums. All categories share the same channel backend
///
/// Implemented for `NCat` chains of `FeagiCategoryRequester`s, and by generated group structs
pub trait FeagiRequestCategoryGroup<Channels>
where
    Channels: FeagiRequestResponseChannelBackend,
{
    /// The requester of every category, as `NCat<FeagiCategoryRequester<A, Channels>, NCat<..., NNull>>`
    type NestedCategoryRequesters;

    /// The configurables of every category, nested the same way as the requesters:
    /// `NCat<&'static A::Configurables, NCat<..., NNull>>`. They live as long as the program
    type NestedConfigurables;

    /// Consume self to get the requester of every category
    fn into_nested_category_requesters(self) -> Self::NestedCategoryRequesters;
}

/// Nests per category items (requesters, configurables) of a group, as `NCat<Head, NCat<..., NNull>>`
pub struct NCat<Head, Tail>
    // Tail can be either another NCat or a NNull (tail)
    {
    pub head: Head,
    pub tail: Tail
}

impl<Channels> FeagiRequestCategoryGroup<Channels> for NNull
where
    Channels: FeagiRequestResponseChannelBackend,
{
    type NestedCategoryRequesters = NNull;
    type NestedConfigurables = NNull;

    fn into_nested_category_requesters(self) -> Self::NestedCategoryRequesters {
        self
    }
}

impl<Category, Channels, Tail> FeagiRequestCategoryGroup<Channels> for NCat<FeagiCategoryRequester<Category, Channels>, Tail>
where
    Category: FeagiRequestCategory,
    Category::Configurables: 'static,
    Channels: FeagiRequestResponseChannelBackend,
    Tail: FeagiRequestCategoryGroup<Channels>,
{
    type NestedCategoryRequesters = Self;
    type NestedConfigurables = NCat<&'static Category::Configurables, Tail::NestedConfigurables>;

    fn into_nested_category_requesters(self) -> Self::NestedCategoryRequesters {
        self
    }
}

//endregion
