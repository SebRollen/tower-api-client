use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use stream_flatten_iters::TryStreamExt;
use tower_jsonapi_client::pagination::{
    query::{QueryModifier, QueryPaginator},
    PaginatedRequest,
};
use tower_jsonapi_client::{Client, Request, RequestData};

#[derive(Clone)]
struct Data {
    page: usize,
}

impl From<Data> for QueryModifier {
    fn from(s: Data) -> QueryModifier {
        let mut data = HashMap::new();
        data.insert("page".into(), s.page.to_string());
        QueryModifier { data }
    }
}

#[derive(Serialize)]
struct GetPassengers {
    size: usize,
    page: Option<usize>,
}

#[derive(Deserialize, Debug)]
struct Passenger {
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PassengersWrapper {
    total_passengers: usize,
    total_pages: usize,
    data: Vec<Passenger>,
}

impl Request for GetPassengers {
    type Data = Self;
    type Response = PassengersWrapper;

    fn endpoint(&self) -> Cow<str> {
        "/v1/passenger".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Query(self)
    }
}

impl PaginatedRequest for GetPassengers {
    type Data = Data;
    type Paginator = QueryPaginator<Self::Response, Data>;

    fn initial_page(&self) -> Option<Data> {
        self.page.map(|page| Data { page })
    }

    fn paginator(&self) -> Self::Paginator {
        QueryPaginator::new(|prev: Option<&Data>, res: &PassengersWrapper| {
            let max_page = res.total_pages;
            match prev {
                None => Some(Data { page: 1 }),
                Some(x) => {
                    if x.page == max_page {
                        None
                    } else {
                        Some(Data { page: x.page + 1 })
                    }
                }
            }
        })
    }
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let client = Client::new("https://api.instantwebtools.net").show_progress();
    let req = GetPassengers {
        page: None,
        size: 1,
    };

    client
        .send_paginated(&req)
        .map(|maybe_wrapper| maybe_wrapper.map(|wrapper| wrapper.data))
        .try_flatten_iters()
        .take(50)
        .for_each(|_| async move {})
        .await;
}
