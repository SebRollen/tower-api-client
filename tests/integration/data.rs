use crate::utils::{FormHello, JsonHello, NameGreeting, QueryHello};
use serde_json::json;
use tower_jsonapi_client::Client;
use wiremock::matchers::{body_json, body_string, header, method, path, query_param};
use wiremock::{Mock, MockServer, Request as MockRequest, ResponseTemplate};

#[tokio::test]
async fn query() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(query_param("name", "world"))
        .respond_with(|req: &MockRequest| {
            let name = req
                .url
                .query_pairs()
                .find(|(k, _)| k == "name")
                .map(|(_, v)| v)
                .unwrap();
            let body = NameGreeting {
                message: format!("Hello, {}!", name),
            };
            ResponseTemplate::new(200).set_body_json(body)
        })
        .mount(&server)
        .await;

    let response = client
        .send(&QueryHello {
            name: "world".into(),
        })
        .await
        .unwrap();
    assert_eq!(
        response,
        NameGreeting {
            message: "Hello, world!".into(),
        }
    );
}

#[tokio::test]
async fn json() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(body_json(json!({"name": "world"})))
        .respond_with(|_: &MockRequest| {
            let body = NameGreeting {
                message: "Hello, world!".into(),
            };
            ResponseTemplate::new(200).set_body_json(body)
        })
        .mount(&server)
        .await;

    let response = client
        .send(&JsonHello {
            name: "world".into(),
        })
        .await
        .unwrap();
    assert_eq!(
        response,
        NameGreeting {
            message: "Hello, world!".into(),
        }
    );
}

#[tokio::test]
async fn form() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .and(body_string("name=world"))
        .respond_with(|_: &MockRequest| {
            let body = NameGreeting {
                message: "Hello, world!".into(),
            };
            ResponseTemplate::new(200).set_body_json(body)
        })
        .mount(&server)
        .await;

    let response = client
        .send(&FormHello {
            name: "world".into(),
        })
        .await
        .unwrap();
    assert_eq!(
        response,
        NameGreeting {
            message: "Hello, world!".into(),
        }
    );
}
