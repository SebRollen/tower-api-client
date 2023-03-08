use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use tower_jsonapi_client::pagination::query::*;
use tower_jsonapi_client::pagination::*;
use tower_jsonapi_client::{Client, Request, RequestData};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, Request as MockRequest, ResponseTemplate};

#[derive(Clone, Serialize)]
struct PaginationRequest {
    page: Option<usize>,
}

impl From<PaginationRequest> for QueryModifier {
    fn from(s: PaginationRequest) -> QueryModifier {
        let mut data = HashMap::new();
        if let Some(x) = s.page {
            data.insert("page".into(), x.to_string());
        }
        QueryModifier { data }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct PaginationResponse {
    next_page: Option<usize>,
    data: String,
}

impl Request for PaginationRequest {
    type Data = Self;
    type Response = PaginationResponse;

    fn endpoint(&self) -> Cow<str> {
        "/page".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Query(self)
    }
}

impl PaginatedRequest for PaginationRequest {
    type Data = Self;
    type Paginator = QueryPaginator<PaginationResponse, Self>;
    fn paginator(&self) -> Self::Paginator {
        QueryPaginator::new(|_, r: &PaginationResponse| {
            r.next_page.map(|page| Self { page: Some(page) })
        })
    }
    fn initial_page(&self) -> Option<Self> {
        self.page.map(|page| Self { page: Some(page) })
    }
}

#[tokio::test]
async fn initial_pagination() {
    let _ = env_logger::try_init();
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(method("GET"))
        .and(path("/page"))
        .and(query_param("page", "1"))
        .respond_with(|_: &MockRequest| {
            let body = PaginationResponse {
                next_page: Some(2),
                data: "Second!".into(),
            };
            ResponseTemplate::new(200).set_body_json(body)
        })
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/page"))
        .and(query_param("page", "2"))
        .respond_with(|_: &MockRequest| {
            let body = PaginationResponse {
                next_page: None,
                data: "Last!".into(),
            };
            ResponseTemplate::new(200).set_body_json(body)
        })
        .mount(&server)
        .await;

    let mut response = client.send_paginated(&PaginationRequest { page: Some(1) });
    assert_eq!(
        response.next().await.unwrap().unwrap().data,
        "Second!".to_string()
    );
    assert_eq!(
        response.next().await.unwrap().unwrap().data,
        "Last!".to_string()
    );
    assert!(response.next().await.is_none());
}
