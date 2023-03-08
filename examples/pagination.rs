use futures::{StreamExt, TryStreamExt as _};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use stream_flatten_iters::TryStreamExt;
use tower::ServiceBuilder;
use tower_jsonapi_client::pagination::PaginatedRequest;
use tower_jsonapi_client::{Client, Request, RequestData, ServiceExt as _};

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
    type PaginationData = usize;
    fn next_page(&self, prev_page: Option<&usize>, response: &PassengersWrapper) -> Option<usize> {
        match prev_page {
            None => Some(1),
            Some(prev_page) => {
                if prev_page == &response.total_pages {
                    None
                } else {
                    Some(prev_page + 1)
                }
            }
        }
    }

    fn update_request(&mut self, page: &usize) {
        self.page = Some(*page as usize)
    }
}

#[tokio::main]
pub async fn main() {
    let client = ServiceBuilder::new()
        .rate_limit(1, std::time::Duration::from_secs(1))
        .service(Client::new("https://api.instantwebtools.net"));

    let req = GetPassengers {
        page: None,
        size: 1,
    };

    client
        .paginate(req)
        .take(5)
        .map(|maybe_wrapper| maybe_wrapper.map(|wrapper| wrapper.data))
        .try_flatten_iters()
        .try_for_each(|res| async move {
            println!("{}", res.name.unwrap_or_else(|| String::from("No name")));
            Ok(())
        })
        .await
        .unwrap();
}
