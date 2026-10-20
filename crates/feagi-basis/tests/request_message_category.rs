use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use feagi_basis::feagi_message::feagi_message_path::configurables::ConfigurableSlot;
use feagi_basis::feagi_message::feagi_message_path::parameters::MessagePathSlotElement;
use feagi_basis::feagi_message::feagi_message_path::FeagiMessagePath;
use feagi_basis::feagi_message::request::feagi_request_descriptor::FeagiRequestMessageDescriptor;
use feagi_basis::feagi_message::request::feagi_request_message::FeagiRequestMessage;
use feagi_basis::feagi_message::request::feagi_request_message_category::FeagiRequestCategory;
use feagi_basis::feagi_message::request::feagi_request_message_grouping::{NMDes, NNull};
use feagi_basis::feagi_message::request::feagi_request_response_payloads::{FeagiMessageError, FeagiUnexpectedError};

//region Test Category

/// An error type written by hand, used by a message through `error: Type`
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CoffeeMachineError {
    OutOfBeans { beans_needed: u32 },
}

impl core::fmt::Display for CoffeeMachineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "not enough beans")
    }
}

impl core::error::Error for CoffeeMachineError {}

impl FeagiMessageError for CoffeeMachineError {}

define_request_message_category! {
    category_name: "CoffeeShop",
    base_path: "shops/$shop_name",
    category_description: "Requests to a coffee shop",
    configurables: {
        shop_name: 16,
    },
    read: {
        "Order": {
            path: "orders/:order_id<u32>",
            path_item_descriptions: {
                order_id: "Which order to read",
            },
            description: "Read an order",
            payload: {
                include_price: bool,
            },
            response: {
                drink: u32, "Which drink was ordered",
                price: Option<u32>,
            },
            error: {
                OrderNotFound: "No order has that ID",
            },
        },
        "Status": {
            description: "Is the shop open",
            path: "status",
            response: {
                open: bool,
            },
        },
    },
    create: {
        "Order": {
            path: "orders",
            description: "Order a drink",
            payload: {
                drink: u32,
                size: u8,
            },
            response: {
                order_id: u32,
            },
            error: CoffeeMachineError,
        },
        "Receipt": {
            path: "receipts",
            description: "Print a receipt",
            alloc_only: true,
            payload: {
                lines: Vec<String>,
            },
        },
    },
}

//endregion

/// Configurables live as long as the program, so they are leaked here
fn configurables() -> &'static CoffeeShopConfigurables {
    Box::leak(Box::new(CoffeeShopConfigurables {
        shop_name: heapless::String::try_from("downtown").unwrap(),
    }))
}

/// Compiles only if `Actual` and `Expected` are the same type
fn assert_same_type<Actual, Expected>()
where
    Actual: SameAs<Expected>,
{
}

trait SameAs<T> {}

impl<T> SameAs<T> for T {}

#[test]
fn category_marker_holds_category_information() {
    assert_eq!(CoffeeShopRequestCategory::CATEGORY_NAME, "CoffeeShop");
    assert_eq!(CoffeeShopRequestCategory::CATEGORY_DESCRIPTION, "Requests to a coffee shop");
    assert_eq!(CoffeeShopRequestCategory::BASE_PATH, "shops/$shop_name");

    // Template order. The alloc only message only exists when feagi_basis has `alloc`
    #[cfg(feature = "alloc")]
    assert_same_type::<
        <CoffeeShopRequestCategory as FeagiRequestCategory>::NestedMessageDescriptors,
        NMDes<
            ReadOrderFeagiRequestMessageDescriptor,
            NMDes<
                ReadStatusFeagiRequestMessageDescriptor,
                NMDes<CreateOrderFeagiRequestMessageDescriptor, NMDes<CreateReceiptFeagiRequestMessageDescriptor, NNull>>,
            >,
        >,
    >();
    #[cfg(not(feature = "alloc"))]
    assert_same_type::<
        <CoffeeShopRequestCategory as FeagiRequestCategory>::NestedMessageDescriptors,
        NMDes<ReadOrderFeagiRequestMessageDescriptor, NMDes<ReadStatusFeagiRequestMessageDescriptor, NMDes<CreateOrderFeagiRequestMessageDescriptor, NNull>>>,
    >();
}

#[test]
fn full_path_starts_with_base_path_and_names_its_slots() {
    let (shops, shop_name, orders, order_id): (&str, ConfigurableSlot, &str, MessagePathSlotElement<u32>) =
        ReadOrderFeagiRequestMessagePath::FULL_MESSAGE_PATH;
    assert_eq!((shops, shop_name.name, orders, order_id.name), ("shops", "shop_name", "orders", "order_id"));

    assert_same_type::<<ReadOrderFeagiRequestMessagePath as FeagiMessagePath>::Parameters, ReadOrderFeagiRequestMessageParameters>();
    assert_same_type::<<CreateOrderFeagiRequestMessagePath as FeagiMessagePath>::Parameters, ()>();
    assert_same_type::<<CreateOrderFeagiRequestMessagePath as FeagiMessagePath>::Configurables, CoffeeShopConfigurables>();
}

#[test]
fn slotted_path_fills_in_configurables() {
    let (shops, shop_name, orders, order_id): (&str, &str, &str, MessagePathSlotElement<u32>) =
        ReadOrderFeagiRequestMessagePath::to_slotted_parameter_message_path(configurables());
    assert_eq!((shops, shop_name, orders, order_id.name), ("shops", "downtown", "orders", "order_id"));
}

#[test]
fn message_path_is_written_with_its_parameters() {
    let mut path = heapless::String::<64>::new();
    ReadOrderFeagiRequestMessagePath::write_message_path(configurables(), &ReadOrderFeagiRequestMessageParameters { order_id: 7 }, &mut path).unwrap();
    assert_eq!(path.as_str(), "shops/downtown/orders/7");

    let mut path = heapless::String::<64>::new();
    CreateOrderFeagiRequestMessagePath::write_message_path(configurables(), &(), &mut path).unwrap();
    assert_eq!(path.as_str(), "shops/downtown/orders");
}

#[test]
fn message_composes_from_and_decomposes_into_its_specifications() {
    let message = ReadOrderFeagiRequestMessage::new(7, true);
    let (parameters, payload) = message.decompose();
    assert_eq!(parameters.order_id, 7);
    assert!(payload.include_price);

    let message = ReadOrderFeagiRequestMessage::compose(parameters, payload);
    assert_eq!((message.order_id, message.include_price), (7, true));

    let message = CreateOrderFeagiRequestMessage::compose((), CreateOrderFeagiRequestMessagePayload { drink: 2, size: 3 });
    assert_eq!((message.drink, message.size), (2, 3));
}

#[test]
fn messages_use_their_defined_error_types() {
    assert_same_type::<<ReadOrderFeagiRequestMessageDescriptor as FeagiRequestMessageDescriptor>::ResponseError, ReadOrderFeagiRequestMessageError>();
    assert_same_type::<<CreateOrderFeagiRequestMessageDescriptor as FeagiRequestMessageDescriptor>::ResponseError, CoffeeMachineError>();
    assert_same_type::<<ReadStatusFeagiRequestMessageDescriptor as FeagiRequestMessageDescriptor>::ResponseError, FeagiUnexpectedError>();
    #[cfg(feature = "alloc")]
    assert_same_type::<<CreateReceiptFeagiRequestMessageDescriptor as FeagiRequestMessageDescriptor>::ResponsePayload, ()>();
}

#[test]
fn errors_serialize_as_plain_serde_enums() {
    assert_eq!(serde_json::to_string(&ReadOrderFeagiRequestMessageError::OrderNotFound).unwrap(), r#""OrderNotFound""#);
    assert_eq!(serde_json::to_string(&FeagiUnexpectedError::UnexpectedError).unwrap(), r#""UnexpectedError""#);
    assert_eq!(
        serde_json::to_string(&CoffeeMachineError::OutOfBeans { beans_needed: 4 }).unwrap(),
        r#"{"OutOfBeans":{"beans_needed":4}}"#
    );

    let error: ReadOrderFeagiRequestMessageError = serde_json::from_str(r#""OrderNotFound""#).unwrap();
    assert_eq!(error.to_string(), "No order has that ID");
}

#[cfg(feature = "std")]
#[test]
fn category_requests_round_trip_through_the_category_channel() {
    use feagi_basis::parallelism::block_on_thread;
    use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend;

    let (mut requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_category_channels::<CoffeeShopRequestCategory>(2);

    let worker = std::thread::spawn(move || {
        block_on_thread(async move {
            for _ in 0..3 {
                responder
                    .async_process_next(async |request| match request {
                        CoffeeShopRequests::ReadOrder(message) if message.order_id == 7 => {
                            let price = message.include_price.then_some(450);
                            CoffeeShopResponses::ReadOrder(Ok(ReadOrderFeagiRequestMessageResponse { drink: 2, price }))
                        }
                        CoffeeShopRequests::ReadOrder(_) => CoffeeShopResponses::ReadOrder(Err(ReadOrderFeagiRequestMessageError::OrderNotFound)),
                        CoffeeShopRequests::CreateOrder(message) => {
                            CreateOrderFeagiRequestMessageDescriptor::into_category_response(Err(CoffeeMachineError::OutOfBeans {
                                beans_needed: u32::from(message.size),
                            }))
                        }
                        _ => CoffeeShopResponses::Nothing,
                    })
                    .await
                    .unwrap();
            }
        })
    });

    block_on_thread(async {
        let order = requester.request(ReadOrderFeagiRequestMessage::new(7, true)).await.unwrap().unwrap();
        assert_eq!((order.drink, order.price), (2, Some(450)));

        let missing = requester.request(ReadOrderFeagiRequestMessage::new(8, false)).await.unwrap();
        assert!(matches!(missing, Err(ReadOrderFeagiRequestMessageError::OrderNotFound)));

        let created = requester.request(CreateOrderFeagiRequestMessage::new(1, 3)).await.unwrap();
        assert_eq!(created.unwrap_err(), CoffeeMachineError::OutOfBeans { beans_needed: 3 });
    });
    worker.join().unwrap();
}
