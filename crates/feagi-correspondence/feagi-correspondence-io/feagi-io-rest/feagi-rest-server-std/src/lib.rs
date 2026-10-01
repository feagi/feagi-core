use core::net::SocketAddrV4;
use std::thread::JoinHandle;
use thingbuf::mpsc::Receiver;
use ohkami::prelude::*;
use ohkami::{Ohkami};
use feagi_io_basis::feagi_server_endpoint_and_config::{FeagiRestServerStartError, FeagiServerEndpointCommand, FeagiServerEndpointLauncher};
use feagi_io_basis::contracts::request_response::request_response_set::RequestResponseEndpointSenderSet;
use feagi_rest_basis::open_api::OpenApiDocument;
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

        // TODO check if socket is grabbable?


        #[cfg(feature = "feagi-rest-openapi-swagger")]
        async fn open_api_json() -> Response {
            static OPEN_API_BYTES: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();

            Response::OK()
                .with_payload(
                    "application/json",
                    OPEN_API_BYTES.get_or_init(|| {

                        let mut api_doc = OpenApiDocument::new();

                        // TODO macros here expand to add properties to api_doc

                        api_doc
                            .to_json()
                            .unwrap()
                            .as_bytes()
                            .to_vec()
                    }
                    )
                )
        }

        let ohkami_server = Ohkami::new((

            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/swagger-ui".GET(open_api_swagger::redirect_swagger),
            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/swagger-ui/feagi-server.html".GET(open_api_swagger::swagger_html),
            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/swagger-ui/swagger-ui.css".GET(open_api_swagger::swagger_css),
            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/swagger-ui/swagger-ui-bundle.js".GET(open_api_swagger::swagger_js),
            #[cfg(feature = "feagi-rest-openapi-swagger")]
            "/api-docs/openapi.json".GET(open_api_json),

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
                smol::block_on(async { ohkami_server.howl(socket).await; });
            }))
        }

        launch_specific_server(ohkami_server, socket)
    }
}


#[cfg(feature = "feagi-rest-openapi-swagger")]
mod open_api_swagger {
    use ohkami::claw::content::Html;
    use ohkami::{Response};
    use feagi_rest_basis::swagger::{SWAGGER_HTML, SWAGGER_CSS, SWAGGER_JS};

    pub async fn swagger_html() -> Html<&'static str> {
        Html(SWAGGER_HTML)
    }

    pub async fn swagger_css() -> Response {
        Response::OK().with_payload("text/css; charset=UTF-8", SWAGGER_CSS)
    }

    pub async fn swagger_js() -> Response {
        Response::OK().with_payload("application/javascript; charset=UTF-8", SWAGGER_JS)
    }


    pub async fn redirect_swagger() -> Response {
        Response::Found().with_headers(|h| h.location("/swagger-ui/feagi-server.html"))
    }
}