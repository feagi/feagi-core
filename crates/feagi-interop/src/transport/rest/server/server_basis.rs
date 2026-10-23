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
/// `routes`, plus the predefined routes enabled by crate features. `open_api_json` is the
/// document served at `OPEN_API_JSON_PATH`, built with `feagi_open_api_json!`. Without one, that
/// path is answered with 404
pub(crate) fn build_router_with_routes<Routes: RestCategoryRoutes>(
    routes: Routes,
    #[cfg(feature = "open_api_endpoint")] open_api_json: Option<&'static str>,
) -> picoserve::Router<impl picoserve::routing::PathRouter> {
    let router = picoserve::Router::from_service(FeagiRestGroupRouter::new(routes));

    // OpenAPI / Swagger
    #[cfg(feature = "open_api_endpoint")]
    let router = {
        use super::open_api_builder::OPEN_API_JSON_PATH;
        use picoserve::routing::get_service;

        let router = router.route(OPEN_API_JSON_PATH, get_service(OpenApiDocumentService { open_api_json }));

        // Serves the Swagger UI page and the CSS and JS files it loads beside that page.
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

    router
}

/// Answers with the OpenAPI document it holds, or 404 if it holds none
#[cfg(feature = "open_api_endpoint")]
struct OpenApiDocumentService {
    open_api_json: Option<&'static str>,
}

#[cfg(feature = "open_api_endpoint")]
impl<State, PathParameters> picoserve::routing::RequestHandlerService<State, PathParameters> for OpenApiDocumentService {
    async fn call_request_handler_service<R: picoserve::io::Read, W: picoserve::response::ResponseWriter<Error = R::Error>>(
        &self,
        _state: &State,
        _path_parameters: PathParameters,
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        use picoserve::response::{IntoResponse, Response, StatusCode};

        let connection = request.body_connection.finalize().await?;
        match self.open_api_json {
            Some(open_api_json) => Response::ok(open_api_json).with_content_type("application/json").write_to(connection, response_writer).await,
            None => (StatusCode::NOT_FOUND, "No OpenAPI document was given to this server\r\n").write_to(connection, response_writer).await,
        }
    }
}

/// Routes requests to the message they are for, sends them through the message's category
/// channel, and writes back the response
///
/// A message is matched by its path and its REST method together, so messages of different
/// methods may share a path. Read messages take their payload from the URL query, all others
/// from a json body. Responses are json: the response payload on success, or the message's error
pub mod routing {
    use feagi_basis::feagi_message::request::feagi_request_descriptor::FeagiMessageEnum;
    use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryRequester, FeagiRequestCategory, FeagiRequestPayloadDecoder};
    use feagi_basis::feagi_message::request::feagi_request_message_grouping::{FeagiConfiguredCategoryRequester, NCat, NNull};
    use feagi_basis::parallelism::feagi_channel::request_response_channel::channel_backend::FeagiRequestResponseChannelBackend;
    use picoserve::io::Read;
    use picoserve::request::{Path, Request};
    use picoserve::url_encoded::UrlEncodedString;
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

    /// The message method a REST method is for, or `None` if no message uses it
    pub fn message_method(http_method: &str) -> Option<FeagiMessageEnum> {
        match http_method {
            "GET" => Some(FeagiMessageEnum::Read),
            "POST" => Some(FeagiMessageEnum::Create),
            "PUT" => Some(FeagiMessageEnum::Edit),
            "DELETE" => Some(FeagiMessageEnum::Delete),
            "PATCH" => Some(FeagiMessageEnum::Patch),
            _ => None,
        }
    }

    /// Whether a route handled a request, or gives it back for the next route to try
    pub enum RouteOutcome<'r, R: Read, W: ResponseWriter> {
        Handled(Result<ResponseSent, W::Error>),
        NotHandled { request: Request<'r, R>, response_writer: W },
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

            let method = message_method(request.parts.method());
            match self.routes.try_handle(&segments, method, request, response_writer).await {
                RouteOutcome::Handled(result) => result,
                // Only on a miss: was it the method, or the path
                RouteOutcome::NotHandled { request, response_writer } if self.routes.any_path_matches(&segments) => {
                    write_status(request, response_writer, StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").await
                }
                RouteOutcome::NotHandled { request, response_writer } => write_status(request, response_writer, StatusCode::NOT_FOUND, "Not Found").await,
            }
        }
    }

    //endregion

    //region Category

    /// Tries the categories in turn. Implemented for the `NCat` chain of a group's
    /// `FeagiConfiguredCategoryRequester`s
    pub trait RestCategoryRoutes {
        /// Handles the request with the message of `method` at `segments`, if there is one. Each
        /// category finds its message with its generated path matcher, and only the category
        /// that matches awaits anything, so futures nest once per category rather than once per
        /// message
        async fn try_handle<'r, R: Read, W: ResponseWriter<Error = R::Error>>(
            &self,
            segments: &[&str],
            method: Option<FeagiMessageEnum>,
            request: Request<'r, R>,
            response_writer: W,
        ) -> RouteOutcome<'r, R, W>;

        /// Whether any message, of any method, is at `segments`
        fn any_path_matches(&self, segments: &[&str]) -> bool;
    }

    impl RestCategoryRoutes for NNull {
        async fn try_handle<'r, R: Read, W: ResponseWriter<Error = R::Error>>(
            &self,
            _segments: &[&str],
            _method: Option<FeagiMessageEnum>,
            request: Request<'r, R>,
            response_writer: W,
        ) -> RouteOutcome<'r, R, W> {
            RouteOutcome::NotHandled { request, response_writer }
        }

        fn any_path_matches(&self, _segments: &[&str]) -> bool {
            false
        }
    }

    impl<Category, Channels, Tail> RestCategoryRoutes for NCat<FeagiConfiguredCategoryRequester<Category, Channels>, Tail>
    where
        Category: FeagiRequestCategory,
        Channels: FeagiRequestResponseChannelBackend,
        FeagiCategoryRequester<Category, Channels>: Clone,
        Tail: RestCategoryRoutes,
    {
        async fn try_handle<'r, R: Read, W: ResponseWriter<Error = R::Error>>(
            &self,
            segments: &[&str],
            method: Option<FeagiMessageEnum>,
            request: Request<'r, R>,
            response_writer: W,
        ) -> RouteOutcome<'r, R, W> {
            let path_match = method.and_then(|method| Category::match_message_path(self.head.configurables, method, segments));
            match (path_match, method) {
                (Some(path_match), Some(method)) => {
                    RouteOutcome::Handled(respond_to_message::<Category, Channels, R, W>(&self.head.requester, path_match, method, request, response_writer).await)
                }
                // Categories may share a base path, so the rest are still tried
                _ => self.tail.try_handle(segments, method, request, response_writer).await,
            }
        }

        fn any_path_matches(&self, segments: &[&str]) -> bool {
            Category::message_path_exists(self.head.configurables, segments) || self.tail.any_path_matches(segments)
        }
    }

    //endregion

    //region Message

    /// Decodes the matched message's payload from the request, sends the request through the
    /// category channel, and writes back the response
    async fn respond_to_message<Category, Channels, R, W>(
        requester: &FeagiCategoryRequester<Category, Channels>,
        path_match: Category::PathMatch,
        method: FeagiMessageEnum,
        mut request: Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error>
    where
        Category: FeagiRequestCategory,
        Channels: FeagiRequestResponseChannelBackend,
        FeagiCategoryRequester<Category, Channels>: Clone,
        R: Read,
        W: ResponseWriter<Error = R::Error>,
    {
        // Read messages take their payload from the URL query, all others from a json body
        let category_request = if method == FeagiMessageEnum::Read {
            Category::decode_request(path_match, RestPayload::Query(request.parts.query().unwrap_or_default()))
        } else if request.body_connection.content_length() == 0 {
            Category::decode_request(path_match, RestPayload::Empty)
        } else {
            match request.body_connection.body().read_all().await {
                Ok(body) => Category::decode_request(path_match, RestPayload::Json(body)),
                Err(_) => None,
            }
        };
        let Some(category_request) = category_request else {
            return write_status(request, response_writer, StatusCode::BAD_REQUEST, "Request payload could not be parsed").await;
        };

        // A clone has its own reply channel, so concurrent requests to the same category do not
        // share one
        let response = requester.clone().make_request(category_request).await;

        let connection = request.body_connection.finalize().await?;
        match response {
            // TODO per error status codes
            Ok(response) if Category::response_is_error(&response) => {
                Json(response)
                    .into_response()
                    .with_status_code(StatusCode::BAD_REQUEST)
                    .write_to(connection, response_writer)
                    .await
            }
            Ok(response) => Json(response).write_to(connection, response_writer).await,
            Err(channel_error) => {
                (StatusCode::SERVICE_UNAVAILABLE, format_args!("{channel_error}\r\n"))
                    .write_to(connection, response_writer)
                    .await
            }
        }
    }

    /// Where a REST request's payload comes from
    enum RestPayload<'a> {
        /// The URL query, for Read messages
        Query(UrlEncodedString<'a>),
        /// A json body
        Json(&'a [u8]),
        /// No body was sent
        Empty,
    }

    impl FeagiRequestPayloadDecoder for RestPayload<'_> {
        fn decode_payload<Payload: serde::de::DeserializeOwned>(self) -> Option<Payload> {
            match self {
                RestPayload::Query(query) => picoserve::url_encoded::deserialize_form(query).ok().or_else(deserialize_nothing),
                RestPayload::Json(body) => serde_json_core::from_slice_escaped(body, &mut [0; JSON_UNESCAPE_BUFFER_LEN]).ok().map(|(payload, _)| payload),
                RestPayload::Empty => deserialize_nothing(),
            }
        }
    }

    /// The payload of a request that sent none. Only succeeds for `()`
    fn deserialize_nothing<Payload: serde::de::DeserializeOwned>() -> Option<Payload> {
        Payload::deserialize(serde::de::value::UnitDeserializer::<serde::de::value::Error>::new()).ok()
    }

    //endregion

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
