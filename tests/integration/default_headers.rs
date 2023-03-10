use crate::utils::EmptyHello;
use tower::ServiceExt;
use tower_api_client::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn default_header() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let mut headers = HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static("tower-api-client"));
    let client = Client::new(&uri).default_headers(headers);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(header("User-Agent", "tower-api-client"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.oneshot(EmptyHello).await.unwrap();
}
