use crate::utils::EmptyHello;
use tower::ServiceExt;
use tower_jsonapi_client::Client;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn header_auth() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let auth = vec![("key", "k"), ("secret", "s")];
    let client = Client::new(&uri).header_auth(auth);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(header("key", "k"))
        .and(header("secret", "s"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.oneshot(EmptyHello).await.unwrap();
}
