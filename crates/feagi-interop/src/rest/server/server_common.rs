//! Systems (largely) common to both STD and Embassy


/// Request read buffer. Response bodies are written from their static slices.
pub const HTTP_BUFFER_LEN: usize = 2048;


/// Builds the static routes for the server.
pub fn build_router_with_routes() -> picoserve::Router<impl picoserve::routing::PathRouter> {
    use picoserve::routing::PathRouter;


    //region Predefined Route Builders

    #[cfg(feature = "open_api_endpoint")]
    fn route_open_api<R: PathRouter>(router: picoserve::Router<R>)
        -> picoserve::Router<impl PathRouter>
    {
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
    fn route_swagger_ui<R: PathRouter>(router: picoserve::Router<R>)
        -> picoserve::Router<impl PathRouter>
    {
        pub const HTML: &str = include_str!("../../../resources_swagger/feagi-server.html");
        pub const CSS: &[u8] = include_bytes!("../../../resources_swagger/swagger-ui.css");
        pub const JS: &[u8] = include_bytes!("../../../resources_swagger/swagger-ui-bundle.js");
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