//! A coffee shop category of messages, and a responder answering them, shared by the REST server
//! tests

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryRequester, FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::feagi_message::request::feagi_request_message_grouping::{NCat, NNull};
use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend;

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
            description: "Read an order",
            payload: {
                include_price: Option<bool>,
            },
            response: {
                drink: u32,
                price: Option<u32>,
            },
            error: {
                OrderNotFound: "No order has that ID",
            },
        },
        "Status": {
            path: "status",
            description: "Is the shop open",
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
        },
    },
    delete: {
        "Order": {
            path: "orders/:order_id<u32>",
            description: "Cancel an order",
        },
    },
}

pub type Channels = FeagiRequestResponseChannelsAllocBackend;
pub type ShopGroup = NCat<FeagiCategoryRequester<CoffeeShopRequestCategory, Channels>, NNull>;
pub type ShopGroupConfigurables = NCat<&'static CoffeeShopConfigurables, NNull>;

/// Starts a responder answering every coffee shop request on its own thread, and returns the
/// group and configurables to serve. The shop is named `shop_name`
pub fn start_coffee_shop(shop_name: &str) -> (ShopGroup, ShopGroupConfigurables, std::thread::JoinHandle<()>) {
    let (requester, responder) = CoffeeShopRequestCategory::spawn_alloc_requester_responser_pair(4);
    let responder_thread = std::thread::spawn(move || answer_coffee_shop_requests(responder));

    let configurables: &'static CoffeeShopConfigurables = Box::leak(Box::new(CoffeeShopConfigurables {
        shop_name: heapless::String::try_from(shop_name).expect("shop name fits"),
    }));
    (NCat { head: requester, tail: NNull }, NCat { head: configurables, tail: NNull }, responder_thread)
}

/// Answers every coffee shop request, until every requester is gone. Order 7 is the only order
fn answer_coffee_shop_requests(responder: FeagiCategoryResponder<CoffeeShopRequestCategory, Channels>) {
    block_on_thread(async move {
        loop {
            let result = responder
                .async_process_next(async |request| match request {
                    CoffeeShopRequests::ReadOrder(message) if message.order_id == 7 => CoffeeShopResponses::ReadOrder(Ok(ReadOrderFeagiRequestMessageResponse {
                        drink: 2,
                        price: message.include_price.unwrap_or(false).then_some(450),
                    })),
                    CoffeeShopRequests::ReadOrder(_) => CoffeeShopResponses::ReadOrder(Err(ReadOrderFeagiRequestMessageError::OrderNotFound)),
                    CoffeeShopRequests::ReadStatus(_) => CoffeeShopResponses::ReadStatus(Ok(ReadStatusFeagiRequestMessageResponse { open: true })),
                    CoffeeShopRequests::CreateOrder(message) => CoffeeShopResponses::CreateOrder(Ok(CreateOrderFeagiRequestMessageResponse {
                        order_id: 40 + message.drink + u32::from(message.size),
                    })),
                    CoffeeShopRequests::DeleteOrder(_) => CoffeeShopResponses::DeleteOrder(Ok(())),
                    CoffeeShopRequests::Nothing => CoffeeShopResponses::Nothing,
                })
                .await;
            if result.is_err() {
                break;
            }
        }
    });
}
