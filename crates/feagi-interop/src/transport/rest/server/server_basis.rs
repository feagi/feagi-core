//! Systems (largely) common to both STD and Embassy

use feagi_basis::feagi_message::request::feagi_request_message_grouping::FeagiRequestCategoryGroup;
use feagi_basis::parallelism::feagi_channel::request_response_channel::channel_backend::FeagiRequestResponseChannelBackend;
use routing::{FeagiRestGroupRouter, RestCategoryRoutes};

/// The routes serving every category of `Group`, paired with their configurables
pub type GroupRestRoutes<Group, Channels> = <Group as FeagiRequestCategoryGroup<Channels>>::NestedCategories;

/// Request read buffer. Response bodies are written from their static slices.
pub const HTTP_BUFFER_LEN: usize = 2048; // TODO this needs to be not const, different devices may want different buffers

/// Builds a REST server worker. Implemented once per backend (std threads, embassy), each also
/// implementing `FeagiWorkerBuilder` for its spawner. Backend specific settings are set on the
/// implementing struct after construction
pub trait RestServerWorkerBuilder<Channels: FeagiRequestResponseChannelBackend>: Sized {
    /// Creates a builder that will serve on `socket_address`, with backend defaults for
    /// everything else
    fn new(socket_address: core::net::SocketAddr) -> Self;
}

/// Builds the router of the server: an endpoint for every message of every category in
/// `routes`, plus the predefined routes enabled by crate features
pub(crate) fn build_router_with_routes<Routes: RestCategoryRoutes>(routes: Routes) -> picoserve::Router<impl picoserve::routing::PathRouter> {
    use picoserve::routing::PathRouter;

    let router = picoserve::Router::from_service(FeagiRestGroupRouter::new(routes));

    /* // TODO make openapi doc generation compile time again, but for now skip
    /// OpenAPI / Swagger
    #[cfg(feature = "open_api_endpoint")]
    let router = {
        use super::open_api_builder::OPEN_API_JSON_PATH;
        use picoserve::routing::get_service;

        let open_api_json = {
            let mut open_api_json = alloc::string::String::new();
            super::open_api_builder::write_open_api_json(&routes, &mut open_api_json).expect("writing to a String does not fail");
            open_api_json
        };

        let router = router.route(OPEN_API_JSON_PATH, get_service(OpenApiDocumentService { open_api_json }));


        /// Serves the Swagger UI page and the CSS and JS files it loads beside that page.
        #[cfg(feature = "swagger_web_ui")]
        let router = {
            use picoserve::response::File;

            pub const HTML: &str = include_str!("../../../../resources_swagger/feagi-server.html");
            pub const CSS: &[u8] = include_bytes!("../../../../resources_swagger/swagger-ui.css");
            pub const JS: &[u8] = include_bytes!("../../../../resources_swagger/swagger-ui-bundle.js");

            // TODO CDN link on our cloud for deployments where we cannot fit the above data in the library

            const SWAGGER_HTML_PATH: &str = "/swagger-ui/feagi-server.html";
            const SWAGGER_CSS_PATH: &str = "/swagger-ui/swagger-ui.css";
            const SWAGGER_JS_PATH: &str = "/swagger-ui/swagger-ui-bundle.js";

            router
                .route(SWAGGER_HTML_PATH, get_service(File::html(HTML)))
                .route(SWAGGER_CSS_PATH, get_service(File::with_content_type(File::MIME_CSS, CSS)))
                .route(SWAGGER_JS_PATH, get_service(File::with_content_type(File::MIME_JS, JS)))
        };


        router
    };

     */

    router
}

/// Answers with the OpenAPI document it holds
#[cfg(all(feature = "open_api_endpoint", feature = "alloc"))]
struct OpenApiDocumentService {
    open_api_json: alloc::string::String,
}

#[cfg(all(feature = "open_api_endpoint", feature = "alloc"))]
impl<State, PathParameters> picoserve::routing::RequestHandlerService<State, PathParameters> for OpenApiDocumentService {
    async fn call_request_handler_service<R: picoserve::io::Read, W: picoserve::response::ResponseWriter<Error = R::Error>>(
        &self,
        _state: &State,
        _path_parameters: PathParameters,
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        use picoserve::response::IntoResponse;

        picoserve::response::Response::ok(self.open_api_json.as_str())
            .with_content_type("application/json")
            .write_to(request.body_connection.finalize().await?, response_writer)
            .await
    }
}

/// Routes requests to the message they are for, sends them through the message's category
/// channel, and writes back the response
///
/// A message is matched by its path and its REST method together, so messages of different
/// methods may share a path. Read messages take their payload from the URL query, all others
/// from a json body. Responses are json: the response payload on success, or the message's error
pub mod routing {
    use feagi_basis::feagi_message::feagi_message_description::{FeagiFieldDescription, FeagiMessagePathElementDescription};
    use feagi_basis::feagi_message::feagi_message_path::configurables::FeagiMessagePathConfigurables;
    use feagi_basis::feagi_message::feagi_message_path::FeagiMessagePath;
    use feagi_basis::feagi_message::request::feagi_request_descriptor::{FeagiMessageEnum, FeagiRequestMessageDescriptor};
    use feagi_basis::feagi_message::request::feagi_request_message::FeagiRequestMessage;
    use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryRequester, FeagiRequestCategory};
    use feagi_basis::feagi_message::request::feagi_request_message_grouping::{FeagiConfiguredCategoryRequester, NCat, NMDes, NNull};
    use feagi_basis::parallelism::feagi_channel::request_response_channel::channel_backend::FeagiRequestResponseChannelBackend;
    use picoserve::extract::{FromRequest, JsonWithUnescapeBufferSize};
    use picoserve::io::Read;
    use picoserve::request::{Path, Request};
    use picoserve::response::{IntoResponse, Json, ResponseWriter, StatusCode};
    use picoserve::routing::PathRouterService;
    use picoserve::ResponseSent;

    /// Most segments a request path may have. Longer paths are answered with 404
    pub const MAX_PATH_SEGMENTS: usize = 16;
    /// Longest a decoded path segment may be. Paths with longer segments are answered with 404
    pub const MAX_PATH_SEGMENT_LEN: usize = 128;
    /// Space for unescaping strings while parsing a json body
    const JSON_UNESCAPE_BUFFER_LEN: usize = 256;

    /// The REST method of a message
    pub const fn http_method(method: FeagiMessageEnum) -> &'static str {
        match method {
            FeagiMessageEnum::Read => "GET",
            FeagiMessageEnum::Create => "POST",
            FeagiMessageEnum::Edit => "PUT",
            FeagiMessageEnum::Delete => "DELETE",
            FeagiMessageEnum::Patch => "PATCH",
        }
    }

    /// Whether a route handled a request, or gives it back for the next route to try
    pub enum RouteOutcome<'r, R: Read, W: ResponseWriter> {
        Handled(Result<ResponseSent, W::Error>),
        NotHandled {
            request: Request<'r, R>,
            response_writer: W,
            /// Some route had this path, but not this method
            path_matched: bool,
        },
    }

    //region Group

    /// The `PathRouterService` serving every message of every category of a group
    pub struct FeagiRestGroupRouter<Routes> {
        routes: Routes,
    }

    impl<Routes: RestCategoryRoutes> FeagiRestGroupRouter<Routes> {
        pub fn new(routes: Routes) -> Self {
            FeagiRestGroupRouter { routes }
        }
    }

    impl<State, Routes: RestCategoryRoutes> PathRouterService<State> for FeagiRestGroupRouter<Routes> {
        async fn call_path_router_service<R: Read, W: ResponseWriter<Error = R::Error>>(
            &self,
            _state: &State,
            _current_path_parameters: (),
            path: Path<'_>,
            request: Request<'_, R>,
            response_writer: W,
        ) -> Result<ResponseSent, W::Error> {
            // Decoded once, then matched against every message path
            let mut decoded_segments = heapless::Vec::<heapless::String<MAX_PATH_SEGMENT_LEN>, MAX_PATH_SEGMENTS>::new();
            for segment in path.segments() {
                let Ok(segment) = segment.try_into_string::<MAX_PATH_SEGMENT_LEN>() else {
                    return write_status(request, response_writer, StatusCode::NOT_FOUND, "Not Found").await;
                };
                if decoded_segments.push(segment).is_err() {
                    return write_status(request, response_writer, StatusCode::NOT_FOUND, "Not Found").await;
                }
            }
            let segments: heapless::Vec<&str, MAX_PATH_SEGMENTS> = decoded_segments.iter().map(|segment| segment.as_str()).collect();

            match self.routes.try_handle(&segments, request, response_writer).await {
                RouteOutcome::Handled(result) => result,
                RouteOutcome::NotHandled {
                    request,
                    response_writer,
                    path_matched: true,
                } => write_status(request, response_writer, StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").await,
                RouteOutcome::NotHandled {
                    request,
                    response_writer,
                    path_matched: false,
                } => write_status(request, response_writer, StatusCode::NOT_FOUND, "Not Found").await,
            }
        }
    }

    //endregion

    //region Category

    /// One route of the server, described for documentation (such as OpenAPI)
    #[derive(Clone, Copy)]
    pub struct RouteDescription<'a> {
        pub category_name: &'static str,
        pub category_description: &'static str,
        pub method: FeagiMessageEnum,
        pub description: &'static str,
        pub path_elements: &'static [FeagiMessagePathElementDescription],
        /// Values of the configurable path elements
        pub configurables: &'a dyn FeagiMessagePathConfigurables,
        /// From the URL query for Read messages, otherwise from the json body
        pub request_payload_fields: &'static [FeagiFieldDescription],
        pub response_payload_fields: &'static [FeagiFieldDescription],
    }

    /// Tries the routes of each category in turn. Implemented for the `NCat` chain of a group's
    /// `FeagiConfiguredCategoryRequester`s
    pub trait RestCategoryRoutes {
        async fn try_handle<'r, R: Read, W: ResponseWriter<Error = R::Error>>(
            &self,
            segments: &[&str],
            request: Request<'r, R>,
            response_writer: W,
        ) -> RouteOutcome<'r, R, W>;

        /// Describes route `index`, counting across all categories in order, or `None` past
        /// the last route
        fn route_description(&self, index: usize) -> Option<RouteDescription<'_>>;
    }

    impl RestCategoryRoutes for NNull {
        fn route_description(&self, _index: usize) -> Option<RouteDescription<'_>> {
            None
        }

        async fn try_handle<'r, R: Read, W: ResponseWriter<Error = R::Error>>(
            &self,
            _segments: &[&str],
            request: Request<'r, R>,
            response_writer: W,
        ) -> RouteOutcome<'r, R, W> {
            RouteOutcome::NotHandled {
                request,
                response_writer,
                path_matched: false,
            }
        }
    }

    impl<Category, Channels, Tail> RestCategoryRoutes for NCat<FeagiConfiguredCategoryRequester<Category, Channels>, Tail>
    where
        Category: FeagiRequestCategory,
        Category::NestedMessageDescriptors: RestMessageRoutes<Category, Channels>,
        Channels: FeagiRequestResponseChannelBackend,
        Tail: RestCategoryRoutes,
    {
        async fn try_handle<'r, R: Read, W: ResponseWriter<Error = R::Error>>(
            &self,
            segments: &[&str],
            request: Request<'r, R>,
            response_writer: W,
        ) -> RouteOutcome<'r, R, W> {
            let outcome = <Category::NestedMessageDescriptors as RestMessageRoutes<Category, Channels>>::try_handle(
                &self.head.requester,
                self.head.configurables,
                segments,
                request,
                response_writer,
            )
            .await;
            let RouteOutcome::NotHandled {
                request,
                response_writer,
                path_matched,
            } = outcome
            else {
                return outcome;
            };
            with_path_matched(self.tail.try_handle(segments, request, response_writer).await, path_matched)
        }

        fn route_description(&self, index: usize) -> Option<RouteDescription<'_>> {
            let route_count = <Category::NestedMessageDescriptors as RestMessageRoutes<Category, Channels>>::ROUTE_COUNT;
            if index < route_count {
                return <Category::NestedMessageDescriptors as RestMessageRoutes<Category, Channels>>::route_description(self.head.configurables, index);
            }
            self.tail.route_description(index - route_count)
        }
    }

    //endregion

    //region Message

    /// Tries the route of each message of a category in turn. Implemented for the `NMDes`
    /// chain of the category
    pub trait RestMessageRoutes<Category, Channels>
    where
        Category: FeagiRequestCategory,
        Channels: FeagiRequestResponseChannelBackend,
    {
        async fn try_handle<'r, R: Read, W: ResponseWriter<Error = R::Error>>(
            requester: &FeagiCategoryRequester<Category, Channels>,
            configurables: &Category::Configurables,
            segments: &[&str],
            request: Request<'r, R>,
            response_writer: W,
        ) -> RouteOutcome<'r, R, W>;

        /// How many messages, and so routes, the chain has
        const ROUTE_COUNT: usize;

        /// Describes the route of message `index` of the chain, or `None` past the last one
        fn route_description(configurables: &Category::Configurables, index: usize) -> Option<RouteDescription<'_>>;
    }

    impl<Category, Channels> RestMessageRoutes<Category, Channels> for NNull
    where
        Category: FeagiRequestCategory,
        Channels: FeagiRequestResponseChannelBackend,
    {
        const ROUTE_COUNT: usize = 0;

        fn route_description(_configurables: &Category::Configurables, _index: usize) -> Option<RouteDescription<'_>> {
            None
        }

        async fn try_handle<'r, R: Read, W: ResponseWriter<Error = R::Error>>(
            _requester: &FeagiCategoryRequester<Category, Channels>,
            _configurables: &Category::Configurables,
            _segments: &[&str],
            request: Request<'r, R>,
            response_writer: W,
        ) -> RouteOutcome<'r, R, W> {
            RouteOutcome::NotHandled {
                request,
                response_writer,
                path_matched: false,
            }
        }
    }

    impl<Descriptor, Tail, Category, Channels> RestMessageRoutes<Category, Channels> for NMDes<Descriptor, Tail>
    where
        Descriptor: FeagiRequestMessageDescriptor<Category = Category>,
        Tail: RestMessageRoutes<Category, Channels>,
        Category: FeagiRequestCategory,
        Channels: FeagiRequestResponseChannelBackend,
        FeagiCategoryRequester<Category, Channels>: Clone,
    {
        async fn try_handle<'r, R: Read, W: ResponseWriter<Error = R::Error>>(
            requester: &FeagiCategoryRequester<Category, Channels>,
            configurables: &Category::Configurables,
            segments: &[&str],
            request: Request<'r, R>,
            response_writer: W,
        ) -> RouteOutcome<'r, R, W> {
            let Some(parameters) = Descriptor::MessagePath::parse_message_path(configurables, segments.iter().copied()) else {
                return Tail::try_handle(requester, configurables, segments, request, response_writer).await;
            };
            if request.parts.method() != http_method(Descriptor::METHOD) {
                let outcome = Tail::try_handle(requester, configurables, segments, request, response_writer).await;
                return with_path_matched(outcome, true);
            }
            RouteOutcome::Handled(respond_to_message::<Descriptor, Category, Channels, R, W>(requester, parameters, request, response_writer).await)
        }

        const ROUTE_COUNT: usize = 1 + Tail::ROUTE_COUNT;

        fn route_description(configurables: &Category::Configurables, index: usize) -> Option<RouteDescription<'_>> {
            if index != 0 {
                return Tail::route_description(configurables, index - 1);
            }
            Some(RouteDescription {
                category_name: Category::CATEGORY_NAME,
                category_description: Category::CATEGORY_DESCRIPTION,
                method: Descriptor::METHOD,
                description: Descriptor::MESSAGE_DESCRIPTION,
                path_elements: <Descriptor::MessagePath as FeagiMessagePath>::PATH_ELEMENTS,
                configurables,
                request_payload_fields: Descriptor::REQUEST_PAYLOAD_FIELDS,
                response_payload_fields: Descriptor::RESPONSE_PAYLOAD_FIELDS,
            })
        }
    }

    /// Builds the message from the request, sends it through the category channel, and writes
    /// back the response
    async fn respond_to_message<Descriptor, Category, Channels, R, W>(
        requester: &FeagiCategoryRequester<Category, Channels>,
        parameters: Descriptor::Parameters,
        mut request: Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error>
    where
        Descriptor: FeagiRequestMessageDescriptor<Category = Category>,
        Category: FeagiRequestCategory,
        Channels: FeagiRequestResponseChannelBackend,
        FeagiCategoryRequester<Category, Channels>: Clone,
        R: Read,
        W: ResponseWriter<Error = R::Error>,
    {
        let Some(request_payload) = decode_request_payload::<<Descriptor::Message as FeagiRequestMessage>::DescriptorRequestPayload, R>(
            Descriptor::METHOD,
            &mut request,
        )
        .await
        else {
            return write_status(request, response_writer, StatusCode::BAD_REQUEST, "Request payload could not be parsed").await;
        };
        let message = Descriptor::Message::compose(parameters, request_payload);

        // A clone has its own reply channel, so concurrent requests to the same category do not
        // share one
        let response = requester.clone().request(message).await;

        let connection = request.body_connection.finalize().await?;
        match response {
            Ok(Ok(response_payload)) => Json(response_payload).write_to(connection, response_writer).await,
            // TODO per error status codes
            Ok(Err(message_error)) => {
                Json(message_error)
                    .into_response()
                    .with_status_code(StatusCode::BAD_REQUEST)
                    .write_to(connection, response_writer)
                    .await
            }
            Err(channel_error) => {
                (StatusCode::SERVICE_UNAVAILABLE, format_args!("{channel_error}\r\n"))
                    .write_to(connection, response_writer)
                    .await
            }
        }
    }

    /// Read messages take their payload from the URL query, all others from a json body. A
    /// payload of `()` may also be left out entirely
    async fn decode_request_payload<Payload: serde::de::DeserializeOwned, R: Read>(method: FeagiMessageEnum, request: &mut Request<'_, R>) -> Option<Payload> {
        if matches!(method, FeagiMessageEnum::Read) {
            let query = request.parts.query().unwrap_or_default();
            return picoserve::url_encoded::deserialize_form(query).ok().or_else(deserialize_nothing);
        }

        if request.body_connection.content_length() == 0 {
            return deserialize_nothing();
        }
        let parts = request.parts;
        <JsonWithUnescapeBufferSize<Payload, JSON_UNESCAPE_BUFFER_LEN> as FromRequest<'_, (), Payload>>::from_request(&(), parts, request.body_connection.body())
            .await
            .ok()
            .map(|JsonWithUnescapeBufferSize(payload)| payload)
    }

    /// The payload of a request that sent none. Only succeeds for `()`
    fn deserialize_nothing<Payload: serde::de::DeserializeOwned>() -> Option<Payload> {
        Payload::deserialize(serde::de::value::UnitDeserializer::<serde::de::value::Error>::new()).ok()
    }

    //endregion

    /// Marks a not handled outcome as having matched the path, if `path_matched`
    fn with_path_matched<'r, R: Read, W: ResponseWriter>(outcome: RouteOutcome<'r, R, W>, path_matched: bool) -> RouteOutcome<'r, R, W> {
        match outcome {
            RouteOutcome::NotHandled {
                request,
                response_writer,
                path_matched: matched_later,
            } => RouteOutcome::NotHandled {
                request,
                response_writer,
                path_matched: path_matched || matched_later,
            },
            handled => handled,
        }
    }

    /// Answers with only a status and a short message
    async fn write_status<R: Read, W: ResponseWriter<Error = R::Error>>(
        request: Request<'_, R>,
        response_writer: W,
        status_code: StatusCode,
        message: &'static str,
    ) -> Result<ResponseSent, W::Error> {
        (status_code, format_args!("{message}\r\n"))
            .write_to(request.body_connection.finalize().await?, response_writer)
            .await
    }
}
