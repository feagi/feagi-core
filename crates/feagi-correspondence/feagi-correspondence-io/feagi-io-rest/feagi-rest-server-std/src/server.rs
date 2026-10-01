use core::net::SocketAddrV4;
use std::thread::JoinHandle;
use thingbuf::mpsc::Receiver;
use ohkami::prelude::*;
use ohkami::claw::content::Html;
use ohkami::{openapi, Ohkami};
use feagi_io_basis::feagi_server_endpoint_and_config::{FeagiFailStartRestServerEtc, FeagiRestServerStartError, FeagiServerEndpointCommand, FeagiServerEndpointLauncher};
use feagi_io_basis::contracts::request_response::request_response_set::RequestResponseEndpointSenderSet;
use feagi_rest_basis::rest_feagi_server_endpoint_config::RestFeagiServerEndpointConfig;


pub struct OhkamiServerEndpointLauncher;

impl<Requesters: RequestResponseEndpointSenderSet>
FeagiServerEndpointLauncher<RestFeagiServerEndpointConfig<Requesters>>
for OhkamiServerEndpointLauncher
{
    fn launch_server_thread(
        config: RestFeagiServerEndpointConfig<Requesters>,
        command_channel: Receiver<FeagiServerEndpointCommand>)
        -> Result<JoinHandle<()>, FeagiRestServerStartError> {



        let socket = config.get_address();
        let requests = config.unwrap_requester();

        let ohkami_server = Ohkami::new((

            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/swagger-ui".GET(redirect_swagger),
            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/swagger-ui/feagi-server.html".GET(swagger_html),
            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/swagger-ui/swagger-ui.css".GET(swagger_css),
            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/swagger-ui/swagger-ui-bundle.js".GET(swagger_js),
            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/api-docs/openapi.json".GET(openapi_json),

            // TODO macro adds other paths here, compiles the data down to a request, sends it to the corresponding Self::RequestModules endpoint
        ));

        #[cfg(not(any(feature = "smol")))]
        return Err(
            FeagiFailStartRestServerEtc::new(
                "Cannot launch ohkami web server with no async runtime feature enabled!"
            )
        );

        // TODO features should also check that other asyncs arent on

        #[cfg(feature = "smol")]
        fn launch_specific_server(ohkami_server: Ohkami, socket: SocketAddrV4) -> Result<JoinHandle<()>, FeagiRestServerStartError> {
            use smol;

            // TODO early return if socket is taken

            Ok(std::thread::spawn(move || {
                smol::block_on(async {
                    ohkami_server.howl(socket).await;
                });
            }))
        }

        launch_specific_server(ohkami_server, socket)
    }
}



mod open_api_swagger {
    use ohkami::claw::content::Html;
    use ohkami::{openapi, Response};

    fn openapi_json_bytes() -> &'static [u8] {
        static SPEC: OnceLock<Vec<u8>> = OnceLock::new();
        SPEC.get_or_init(|| create_ohkami_server().__openapi_document_bytes__(rest_openapi_metadata()))
    }


    async fn swagger_html() -> Html<&'static str> {
        Html(SWAGGER_HTML)
    }

    async fn swagger_css() -> Response {
        Response::OK().with_payload("text/css; charset=UTF-8", SWAGGER_CSS)
    }

    async fn swagger_js() -> Response {
        Response::OK().with_payload("application/javascript; charset=UTF-8", SWAGGER_JS)
    }

    async fn openapi_json() -> Response {
        Response::OK().with_payload("application/json", openapi_json_bytes())
    }

    async fn redirect_swagger() -> Response {
        Response::Found().with_headers(|h| h.location("/swagger-ui/feagi-server.html"))
    }

    fn rest_openapi_metadata() -> openapi::OpenAPI<'static> {
        openapi::OpenAPI {
            title: "FEAGI REST Server",
            version: env!("CARGO_PKG_VERSION"),
            servers: &[],
        }
    }
}








//endregion

