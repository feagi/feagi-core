use std::borrow::Cow;

use zenoh::bytes::Encoding;
use zenoh::config::Config;

// =========================================================================
// Compile-time Static Asset Inclusion
// =========================================================================
static INDEX_HTML: &str = include_str!("../resources_swagger/feagi-server.html");
static SWAGGER_CSS: &str = include_str!("../resources_swagger/swagger-ui.css");
static SWAGGER_JS: &str = include_str!("../resources_swagger/swagger-ui-bundle.js");
//static OPENAPI_JSON: &str = include_str!("../resources_swagger/openapi.json");

#[tokio::main]
async fn main() {
    println!("Launching Monolithic Single-Port Zenoh Gateway...");


    let mut config = Config::default();

    config.insert_json5("mode", r#""router""#).unwrap();

    config.insert_json5("plugins_loading/enabled", "true").unwrap();
    // A bare port binds [::]:8000. On Windows that socket does not accept 127.0.0.1.
    config
        .insert_json5("plugins/rest/http_port", r#""0.0.0.0:8000""#)
        .unwrap();
    config.insert_json5("plugins/rest/__required__", "true").unwrap();

    let session = zenoh::open(config).await.unwrap();

    let file_queryable = session.declare_queryable("docs/**").complete(true).await.unwrap();

    tokio::spawn(async move {
        while let Ok(query) = file_queryable.recv_async().await {
            let key_expr = query.key_expr().clone();
            let key = key_expr.as_str();

            // Match the end of the URL path to serve the correct compiled asset.
            // Zenoh REST plugin returns a JSON sample list unless the
            // request includes `_raw=true`, which returns the payload with its encoding.
            let (content, encoding) = match key {
                k if k.ends_with("index.html") => (
                    Cow::Owned(
                        INDEX_HTML
                            .replace("href=\"swagger-ui.css\"", "href=\"swagger-ui.css?_raw=true\"")
                            .replace("src=\"swagger-ui-bundle.js\"", "src=\"swagger-ui-bundle.js?_raw=true\""),
                    ),
                    Encoding::TEXT_HTML,
                ),
                k if k.ends_with("swagger-ui.css") => (Cow::Borrowed(SWAGGER_CSS), Encoding::TEXT_CSS),
                k if k.ends_with("swagger-ui-bundle.js") => (Cow::Borrowed(SWAGGER_JS), Encoding::TEXT_JAVASCRIPT),
                //k if k.ends_with("openapi.json") => (Cow::Borrowed(OPENAPI_JSON), Encoding::APPLICATION_JSON),
                _ => {
                    let _ = query.reply_err("404 Not Found").await;
                    continue;
                }
            };

            // Wrap payload and send back down the REST wire
            let _ = query.reply(key_expr, content.as_ref()).encoding(encoding).await;
        }
    });

    let api_queryable = session.declare_queryable("api/v1/status").complete(true).await.unwrap();
    println!("Monolith running. Open UI at: http://localhost:8000/docs/index.html?_raw=true");

    while let Ok(query) = api_queryable.recv_async().await {
        println!("Swagger UI executed a live API check on: {}", query.key_expr().as_str());

        let json_response = r#"{"status": "online", "architecture": "monolithic-single-port"}"#;
        let _ = query
            .reply(query.key_expr().clone(), json_response)
            .encoding(Encoding::APPLICATION_JSON)
            .await;
    }
}
