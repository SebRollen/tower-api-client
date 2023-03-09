use crate::utils::EmptyHello;
use tower::ServiceExt;
use tower_jsonapi_client::Client;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn empty_response() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.oneshot(EmptyHello).await.unwrap();
}
