use crate::utils::EmptyHello;
use tower_jsonapi_client::{Client, Error, StatusCode};
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn client_error() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(any())
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    assert!(matches!(
        client.send(&EmptyHello).await.unwrap_err(),
        Error::ClientError(status, msg) if (status == StatusCode::NOT_FOUND && msg == String::new())
    ));
}

#[tokio::test]
async fn server_error() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(any())
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    assert!(matches!(
        client.send(&EmptyHello).await.unwrap_err(),
        Error::ServerError(status, msg) if (status == StatusCode::INTERNAL_SERVER_ERROR && msg == String::new())
    ));
}
