use serde::Serialize;
use serde_json::json;
use std::borrow::Cow;
use tower::ServiceExt;
use tower_api_client::{Client, EmptyResponse, Method, Request, RequestData};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[derive(Serialize)]
struct CreateUser {
    name: String,
}

impl Request for CreateUser {
    type Data = Self;
    type Response = EmptyResponse;
    const METHOD: Method = Method::POST;

    fn endpoint(&self) -> Cow<str> {
        "/user".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Json(self)
    }
}

#[tokio::test]
async fn post() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(method("POST"))
        .and(path("/user"))
        .and(body_json(json!({"name": "User"})))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client
        .oneshot(CreateUser {
            name: "User".into(),
        })
        .await
        .unwrap();
}
