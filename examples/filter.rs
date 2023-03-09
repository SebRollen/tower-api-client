use futures::StreamExt;
use serde::Serialize;
use std::borrow::Cow;
use tower::ServiceBuilder;
use tower_api_client::{
    pagination::PaginatedRequest, Client, EmptyResponse, Request, RequestData, ServiceExt,
};

#[derive(Clone, Serialize)]
struct GetPassengers {
    size: usize,
    page: Option<usize>,
}

impl Request for GetPassengers {
    type Data = Self;
    type Response = EmptyResponse;

    fn endpoint(&self) -> Cow<str> {
        "/v1/passenger".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Query(self)
    }
}

impl PaginatedRequest for GetPassengers {
    type PaginationData = usize;
    fn next_page(&self, prev_page: Option<&usize>, _response: &EmptyResponse) -> Option<usize> {
        Some(prev_page.unwrap_or(&0) + 1)
    }

    fn update_request(&mut self, page: &usize) {
        self.page = Some(*page as usize)
    }
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let client = ServiceBuilder::new()
        .filter(|req: GetPassengers| {
            if req.page.unwrap_or(0) < 4 {
                Ok(req)
            } else {
                Err("Not allowed!")
            }
        })
        .rate_limit(1, std::time::Duration::from_secs(2))
        .service(Client::new("https://api.instantwebtools.net"));

    let req = GetPassengers {
        size: 10,
        page: None,
    };
    client
        .paginate(req)
        .take(5)
        .enumerate()
        .for_each(|(i, _)| async move { println!("{}", i) })
        .await;
}
