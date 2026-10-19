//! Systems (largely) common to both STD and Embassy

/// Request read buffer. Response bodies are written from their static slices.
pub const HTTP_BUFFER_LEN: usize = 2048;




/// Builds a REST server worker. Implemented once per backend (std threads, embassy), each also
/// implementing `FeagiWorkerBuilder` for its spawner. Backend specific settings are set on the
/// implementing struct after construction
pub trait RestServerWorkerBuilder<>: Sized {
    /// Creates a builder that will serve on `socket_address`, with backend defaults for
    /// everything else
    fn new(socket_address: core::net::SocketAddr) -> Self;

    /// Builds the static routes for the server.
    fn build_router_with_routes<>() -> picoserve::Router<impl picoserve::routing::PathRouter> {
        use picoserve::routing::PathRouter;

        //region Predefined Route Builders

        #[cfg(feature = "open_api_endpoint")]
        fn route_open_api<R: PathRouter>(router: picoserve::Router<R>) -> picoserve::Router<impl PathRouter> {
            use picoserve::response::File;
            use picoserve::routing::get_service;

            /// Generated OpenAPI json
            const OPEN_API_JSON: &str = {
                use super::open_api_builder::OpenApiDocument;

                const DOCUMENT: OpenApiDocument = OpenApiDocument::from_categories(&[]);
                const JSON_LEN: usize = DOCUMENT.json_len();
                const JSON_BYTES: [u8; JSON_LEN] = DOCUMENT.write_json();
                OpenApiDocument::json_str(&JSON_BYTES)
            };

            const OPEN_API_JSON_PATH: &str = "/api-docs/openapi.json";

            router.route(
                OPEN_API_JSON_PATH,
                get_service(File::with_content_type("application/json", OPEN_API_JSON.as_bytes())),
            )
        }

        /// Serves the Swagger UI page and the CSS and JS files it loads beside that page.
        #[cfg(feature = "swagger_web_ui")]
        fn route_swagger_ui<R: PathRouter>(router: picoserve::Router<R>) -> picoserve::Router<impl PathRouter> {
            pub const HTML: &str = include_str!("../../../../resources_swagger/feagi-server.html");
            pub const CSS: &[u8] = include_bytes!("../../../../resources_swagger/swagger-ui.css");
            pub const JS: &[u8] = include_bytes!("../../../../resources_swagger/swagger-ui-bundle.js");
            // TODO CDN link on our cloud for deployments where we cannot fit the above data in the library

            use picoserve::response::File;
            use picoserve::routing::get_service;

            const SWAGGER_HTML_PATH: &str = "/swagger-ui/feagi-server.html";
            const SWAGGER_CSS_PATH: &str = "/swagger-ui/swagger-ui.css";
            const SWAGGER_JS_PATH: &str = "/swagger-ui/swagger-ui-bundle.js";

            router
                .route(SWAGGER_HTML_PATH, get_service(File::html(HTML)))
                .route(SWAGGER_CSS_PATH, get_service(File::with_content_type(File::MIME_CSS, CSS)))
                .route(SWAGGER_JS_PATH, get_service(File::with_content_type(File::MIME_JS, JS)))
        }

        //endregion

        //let _pending_route_definitions = PhantomData::<RequestPathDefinition>;

        let router = picoserve::Router::new();

        #[cfg(feature = "open_api_endpoint")]
        let router = route_open_api(router);

        #[cfg(feature = "swagger_web_ui")]
        let router = route_swagger_ui(router);

        // TODO Main routes

        router
    }
}


pub mod path_builder {
    use picoserve::routing::{get, PathDescription, PathRouter, parse_path_segment, ParsePathSegment};
    use feagi_basis::feagi_message::feagi_message_components::{MessageQueryables, FeagiMessageParameterFieldSerializable};
    use feagi_basis::feagi_message::feagi_message_path::{FeagiRequestMessagePath, PathParameterElement};
    use feagi_basis::feagi_message::feagi_request_message_grouping::RequestCategoryChannelRouter;
    use feagi_basis::parallelism::feagi_channel::feagi_channel::{FeagiChannelReceiver, FeagiChannelSender};
    use feagi_basis::parallelism::feagi_channel::recycling::Recyclable;
    use feagi_basis::parallelism::feagi_channel::request_response::feagi_request_response::{FeagiRequester, ReplySlot, RequestSlot};
    use feagi_interop_proc::generate_into_picoserve_path_impls;

    pub(crate) struct PicoServeRequestCategoryChannelRouter<Cat, Request, Response, ReplySender, RequestSender, ReplyReceiver>
    where
        Cat: RequestCategoryChannelRouter<Request, Response, ReplySender, RequestSender, ReplyReceiver>,
        Request: Recyclable,
        Response: Recyclable,
        ReplySender: FeagiChannelSender<ReplySlot<Response>> + Clone,
        RequestSender: FeagiChannelSender<RequestSlot<Request, ReplySender>>,
        ReplyReceiver: FeagiChannelReceiver<ReplySlot<Response>>
    {
        requester: FeagiRequester<
            Request, Response,
            RequestSender, ReplySender, ReplyReceiver
        >,
        _p: core::marker::PhantomData<Cat>
    }

    impl<Cat, Request, Response, ReplySender, RequestSender, ReplyReceiver> PicoServeRequestCategoryChannelRouter<Cat, Request, Response, ReplySender, RequestSender, ReplyReceiver>
    where
        Cat: RequestCategoryChannelRouter<Request, Response, ReplySender, RequestSender, ReplyReceiver>,
        Request: Recyclable,
        Response: Recyclable,
        ReplySender: FeagiChannelSender<ReplySlot<Response>> + Clone,
        RequestSender: FeagiChannelSender<RequestSlot<Request, ReplySender>>,
        ReplyReceiver: FeagiChannelReceiver<ReplySlot<Response>>
    {

        pub fn new(requester: FeagiRequester<Request, Response, RequestSender, ReplySender, ReplyReceiver>) -> Self {
            Self {requester, _p: core::marker::PhantomData}
        }

        pub fn unwrap_requester(self) -> FeagiRequester<Request, Response, RequestSender, ReplySender, ReplyReceiver> {
            self.requester
        }

        pub fn apply_routes<R>(router: picoserve::Router<R>) where R: PathRouter {

        }
    }

    //region Individual Messages

    /// Adds a path to the picoserve router. Needs the queryables specifically.
    ///
    /// The path's type tuple is converted to a picoserve path description through
    /// [`IntoPicoservePath`]. The `NewPathParameters = ()` bound matches the
    /// current zero-argument placeholder handler and will be generalised once
    /// parameter/queryable handling is implemented.
    pub fn add_request_path_to_pico_serve_router<R, Path, Q>(
        router: picoserve::Router<R>,
        path_configurables: Path::ConfigurablesTuple,
        message_queryables: Q,
    ) -> picoserve::Router<impl PathRouter>
    where
        R: PathRouter,
        Path: FeagiRequestMessagePath,
        Path::TypesTuple: IntoPicoservePath,
        <Path::TypesTuple as IntoPicoservePath>::Output: PathDescription<(), NewPathParameters = ()>,
        Q: MessageQueryables,
    {
        let tuple_path = Path::as_path_parser_tuple(path_configurables);
        let picoserve_tuple = tuple_path.into_picoserve_path();
        router.route(
            picoserve_tuple,
            get(async || "test_response"), // TODO queryables
        )
    }

    /// Converts a single FEAGI path element into its picoserve path-segment type.
    trait IntoPicoserveSegment {
        /// The picoserve segment type this element becomes.
        type Output;

        /// Produce the picoserve segment for this element.
        fn into_picoserve_segment(self) -> Self::Output;
    }

    /// A static or configured segment maps straight through as a literal match.
    impl<'a> IntoPicoserveSegment for &'a str {
        type Output = &'a str;

        #[inline]
        fn into_picoserve_segment(self) -> Self::Output {
            self
        }
    }

    /// A typed parameter becomes a `ParsePathSegment<Par>`, which picoserve parses
    /// from the URL segment using `Par: FromStr`
    impl<Par: FeagiMessageParameterFieldSerializable> IntoPicoserveSegment for PathParameterElement<Par> {
        type Output = ParsePathSegment<Par>;

        #[inline]
        fn into_picoserve_segment(self) -> Self::Output {
            parse_path_segment::<Par>()
        }
    }

    /// Converts a whole FEAGI path tuple into the picoserve path-description tuple
    /// accepted by `Router::route`. Per-arity impls are macro-generated.
    trait IntoPicoservePath {
        /// The picoserve path-description tuple produced from `Self`.
        type Output;

        /// Produce the picoserve path description for this tuple.
        fn into_picoserve_path(self) -> Self::Output;
    }

    // One `IntoPicoservePath` impl per tuple arity
    generate_into_picoserve_path_impls!();

    //endregion
}


