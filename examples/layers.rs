use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_api_client::{pagination::PaginatedRequest, Client, Request, RequestData, ServiceExt};

#[derive(Clone, Deserialize, Debug)]
struct Return {}

#[derive(Clone, Serialize)]
struct GetPassengers {
    size: usize,
    page: Option<usize>,
}

impl Request for GetPassengers {
    type Data = Self;
    type Response = Return;

    fn endpoint(&self) -> Cow<str> {
        "/v1/passenger".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Query(self)
    }
}

impl PaginatedRequest for GetPassengers {
    type PaginationData = usize;
    fn get_page(&self) -> Option<usize> {
        self.page
    }
    fn next_page(&self, prev_page: Option<&usize>, _response: &Return) -> Option<usize> {
        match prev_page {
            None => Some(1),
            Some(page) => Some(page + 1),
        }
    }

    fn update_request(&mut self, page: &usize) {
        self.page = Some(*page as usize)
    }
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    // A client that will send one paginated request per second, and error after 4 requests
    let client = ServiceBuilder::new()
        .filter(|req: GetPassengers| {
            if req.page.unwrap_or(0) < 4 {
                Ok(req)
            } else {
                Err("Not allowed!")
            }
        })
        .rate_limit(1, Duration::from_secs(1))
        .service(Client::new("https://api.instantwebtools.net"));

    let req = GetPassengers {
        size: 10,
        page: None,
    };

    client
        .paginate(req)
        .take(5)
        .enumerate()
        // replace value with count
        .map(|(i, x)| x.map(|_| i + 1))
        .try_for_each(|i| async move {
            println!("Request #{}", i);
            Ok(())
        })
        .await
        .unwrap();
}
