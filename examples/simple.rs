use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tower::ServiceExt;
use tower_api_client::{Client, Request, RequestData};

#[derive(Serialize)]
struct GetPassengers {
    size: usize,
}

#[derive(Deserialize, Debug)]
struct Passenger {
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PassengersWrapper {
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

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let client = Client::new("https://api.instantwebtools.net");

    let req = GetPassengers { size: 10 };
    let res = client.oneshot(req).await.unwrap();
    res.data
        .iter()
        .for_each(|passenger| println!("{}", passenger.name.as_deref().unwrap_or("No name")));
}
