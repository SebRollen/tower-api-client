use crate::utils::EmptyHello;
use tower_jsonapi_client::Client;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn basic_auth() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri).basic_auth("user", "pass");

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(header("Authorization", "Basic dXNlcjpwYXNz")) // user:pass in base64
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.send(&EmptyHello).await.unwrap();
}

#[tokio::test]
async fn basic_auth_no_password() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri).basic_auth("user", None);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(header("Authorization", "Basic dXNlcjo=")) // user:pass in base64
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.send(&EmptyHello).await.unwrap();
}
