
#[macro_export]
/// Given a list of categories of requests, outputs structs and a function for generating them, for 
/// a bundle of Senders / Receivers meant to handle passing requests and responses
macro_rules! create_request_response_sender_receiver_sets {
    (
        $base_name:ident: {
            $($category_base_name:ident, $request_pool_size:expr, $allow_beyond_pool_allocation:expr, $incoming_request_queue_length: expr),+ $(,)?
        }

    ) => {

        paste::paste! {

            #[derive(Clone)]
            pub struct [<$base_name:pascal SenderSet>] {
                $(
                    pub $category_base_name: PooledOneshotRequester< [<$category_base_name:pascal RequestsEnum>], [<$category_base_name:pascal ResponsesEnum>], RequestResponseRecycle, RequestResponseRecycle, [< FeagiRequestReceive $category_base_name:pascal Error>], $request_pool_size, $allow_beyond_pool_allocation >,
                )+
            }
            
            impl RequestResponseEndpointSenderSet for [<$base_name:pascal SenderSet>] {}

            

            pub struct [<$base_name:pascal ReceiverSet>] {
                $(
                    pub $category_base_name: RequestResponder< [<$category_base_name:pascal RequestsEnum>], [<$category_base_name:pascal ResponsesEnum>], RequestResponseRecycle, RequestResponseRecycle, [< FeagiRequestReceive $category_base_name:pascal Error>]>,
                )+
            }
            
            impl RequestResponseEndpointReceiverSet for [<$base_name:pascal ReceiverSet>] {}

            
            
            pub fn [<create_request_response_endpoint_set_ $base_name:snake>] -> ([<$base_name:pascal SenderSet>], [<$base_name:pascal ReceiverSet>]) {
                $(
                    let ([<$category_base_name _requester>], [<$category_base_name _responder>]) = [<create_requester_responder_ $category_base_name>]<$request_pool_size, $allow_beyond_pool_allocation>($incoming_request_queue_length);
                )+

                let sender = [<$base_name:pascal SenderSet>] {
                    $($category_base_name: [<$category_base_name _requester>],)+
                }

                let receiver = [<$base_name:pascal ReceiverSet>] {
                    $($category_base_name: [<$category_base_name _responder>],)+
                }

                (sender, receiver)
            }

        }

    };
}



/// Marker trait representing some set of Senders for a given category. Impls of this
/// trait are macro generated, where the member name is the name of the category of request
/// and the value is the Sender<(Result<Response, Error>, Oneshot)> to use. Must be clonable as
/// multiple systems may be able to send requests to a single request processor
pub trait RequestResponseEndpointSenderSet: Clone {}

/// Marker trait representing some set of Receivers for a given category. Impls of this
/// trait are macro generated, where the member name is the name of the category of request
/// and the value is the Request<Result<Response, Error>> to use. Must NOT be clonable as
/// there can only be one output for the multiple
pub trait RequestResponseEndpointReceiverSet {}