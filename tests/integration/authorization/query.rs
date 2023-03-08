use crate::utils::EmptyHello;
use tower_jsonapi_client::Client;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn query_auth() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let auth = vec![("key", "k"), ("secret", "s")];
    let client = Client::new(&uri).query_auth(auth);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(query_param("key", "k"))
        .and(query_param("secret", "s"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.send(EmptyHello).await.unwrap();
}
