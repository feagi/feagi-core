use feagi_basis::feagi_basis_proc_common::{define_request_category_group, define_request_message_category};
use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryRequester, FeagiRequestCategory};
use feagi_basis::feagi_message::request::feagi_request_message_grouping::{FeagiConfiguredCategoryRequester, FeagiRequestCategoryGroup, NCat, NNull};

//region Test Categories

/// A category without configurables
mod tea {
    use super::*;

    define_request_message_category! {
        category_name: "Tea",
        base_path: "tea",
        category_description: "Tea requests",
        read: {
            "Kettle": {
                path: "kettle",
                description: "Is the kettle on",
                response: {
                    on: bool,
                },
            },
        },
    }
}

/// A category with configurables
mod shelf {
    use super::*;

    define_request_message_category! {
        category_name: "Shelf",
        base_path: "shelves/$shelf_name",
        category_description: "Shelf requests",
        configurables: {
            shelf_name: 8,
        },
        read: {
            "Count": {
                path: "count",
                description: "How many items are on the shelf",
                response: {
                    count: u32,
                },
            },
        },
    }
}

use shelf::{ShelfConfigurables, ShelfRequestCategory};
use tea::{ReadKettleFeagiRequestMessage, ReadKettleFeagiRequestMessageResponse, TeaRequestCategory, TeaRequests, TeaResponses};

define_request_category_group! {
    group_name: TeaOnly,
    categories: {
        tea: TeaRequestCategory,
    }
}

define_request_category_group! {
    group_name: Pantry,
    categories: {
        tea: TeaRequestCategory,
        shelf: ShelfRequestCategory,
    }
}

//endregion

/// Compiles only if `Actual` and `Expected` are the same type
fn assert_same_type<Actual, Expected>()
where
    Actual: SameAs<Expected>,
{
}

trait SameAs<T> {}

impl<T> SameAs<T> for T {}

/// Groups whose categories declare no configurables can get theirs in a const
const TEA_ONLY_CONFIGURABLES: TeaOnlyConfigurables = TeaOnlyConfigurables::without_configurables();

#[test]
fn categories_report_whether_they_declare_configurables() {
    assert!(TeaRequestCategory::EMPTY_CONFIGURABLES.is_some());
    assert!(ShelfRequestCategory::EMPTY_CONFIGURABLES.is_none());
    let _ = TEA_ONLY_CONFIGURABLES.tea;
}

#[test]
#[should_panic(expected = "category `shelf` of group `Pantry` declares configurables")]
fn configurables_must_be_given_for_categories_that_declare_them() {
    let _ = PantryConfigurables::without_configurables();
}

#[test]
fn group_nests_each_requester_with_its_configurables() {
    use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend as Channels;

    assert_same_type::<
        <PantryRequesters<Channels> as FeagiRequestCategoryGroup<Channels>>::NestedCategories,
        NCat<
            FeagiConfiguredCategoryRequester<TeaRequestCategory, Channels>,
            NCat<FeagiConfiguredCategoryRequester<ShelfRequestCategory, Channels>, NNull>,
        >,
    >();
    assert_same_type::<<PantryRequesters<Channels> as FeagiRequestCategoryGroup<Channels>>::Configurables, PantryConfigurables>();

    static SHELF: ShelfConfigurables = ShelfConfigurables {
        shelf_name: heapless::String::new(),
    };
    let (requesters, _responders) = PantryRequesters::create_alloc_channel_pairs(2);
    let configurables = PantryConfigurables {
        tea: TeaRequestCategory::EMPTY_CONFIGURABLES.unwrap(),
        shelf: &SHELF,
    };
    let nested = requesters.into_nested_categories(configurables);
    assert!(core::ptr::eq(nested.tail.head.configurables, &SHELF));
}

#[test]
fn joined_groups_nest_as_one_flat_chain() {
    use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend as Channels;

    // A generated group joined with a lone category, as categories served by different workers are
    type Joined = (TeaOnlyRequesters<Channels>, FeagiCategoryRequester<ShelfRequestCategory, Channels>);
    assert_same_type::<
        <Joined as FeagiRequestCategoryGroup<Channels>>::NestedCategories,
        <PantryRequesters<Channels> as FeagiRequestCategoryGroup<Channels>>::NestedCategories,
    >();
    assert_same_type::<<Joined as FeagiRequestCategoryGroup<Channels>>::Configurables, (TeaOnlyConfigurables, &'static ShelfConfigurables)>();

    static SHELF: ShelfConfigurables = ShelfConfigurables {
        shelf_name: heapless::String::new(),
    };
    let (tea, _tea_responders) = TeaOnlyRequesters::create_alloc_channel_pairs(2);
    let (shelf, _shelf_responder) = ShelfRequestCategory::spawn_alloc_requester_responser_pair(2);
    let nested = (tea, shelf).into_nested_categories((TEA_ONLY_CONFIGURABLES, &SHELF));
    assert!(core::ptr::eq(nested.tail.head.configurables, &SHELF));
}

#[cfg(feature = "std")]
#[test]
fn unhandled_categories_are_answered_as_not_implemented() {
    use feagi_basis::parallelism::block_on_thread;

    let (mut requester, responder) = TeaRequestCategory::spawn_alloc_requester_responser_pair(2);
    let worker = std::thread::spawn(move || {
        block_on_thread(async move {
            while responder.try_answer_not_implemented().await.is_err() {
                std::thread::yield_now();
            }
        })
    });

    let kettle = block_on_thread(requester.request(ReadKettleFeagiRequestMessage::new())).unwrap();
    assert!(kettle.is_err());
    worker.join().unwrap();
}

#[cfg(feature = "std")]
#[test]
fn group_requesters_reach_their_responders() {
    use feagi_basis::parallelism::block_on_thread;

    let (requesters, responders) = TeaOnlyRequesters::create_alloc_channel_pairs(2);
    // Requesters are cloned as a whole group
    let mut clone: FeagiCategoryRequester<TeaRequestCategory, _> = requesters.clone().tea;

    let worker = std::thread::spawn(move || {
        block_on_thread(async move {
            responders
                .tea
                .async_process_next(async |request| match request {
                    TeaRequests::ReadKettle(_) => TeaResponses::ReadKettle(Ok(ReadKettleFeagiRequestMessageResponse { on: true })),
                    TeaRequests::Nothing => TeaResponses::Nothing,
                })
                .await
                .unwrap();
        })
    });

    let kettle = block_on_thread(clone.request(ReadKettleFeagiRequestMessage::new())).unwrap().unwrap();
    assert!(kettle.on);
    worker.join().unwrap();
}
