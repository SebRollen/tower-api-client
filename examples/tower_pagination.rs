use futures::{StreamExt, TryStreamExt as _};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use stream_flatten_iters::TryStreamExt;
use tower_jsonapi_client::pagination::PaginatedRequest;
use tower_jsonapi_client::{Client, Request, RequestData, ServiceExt};

#[derive(Clone, Debug, Serialize)]
struct GetPassengers {
    size: usize,
    page: Option<usize>,
}

#[derive(Clone, Deserialize, Debug)]
struct Passenger {
    name: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PassengersWrapper {
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
    type PaginationData = i32;
    fn next_page(&self, prev_page: Option<&i32>, response: &PassengersWrapper) -> Option<i32> {
        match prev_page {
            None => Some(1),
            Some(prev_page) => {
                if prev_page == &(response.total_pages as i32) {
                    None
                } else {
                    Some(prev_page + 1)
                }
            }
        }
    }

    fn update_request(&mut self, page: &i32) {
        self.page = Some(*page as usize)
    }
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let client = Client::new("https://api.instantwebtools.net");

    let req = GetPassengers {
        page: None,
        size: 1,
    };

    client
        .paginate(req)
        .map(|maybe_wrapper| maybe_wrapper.map(|wrapper| wrapper.data))
        .try_flatten_iters()
        .take(5)
        .try_for_each(|res| async move {
            println!("{:?}", res.name);
            Ok(())
        })
        .await
        .unwrap();
}
