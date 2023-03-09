use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tower::{Service, ServiceBuilder};
use tower_jsonapi_client::{Client, Request, RequestData};

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
    let mut client = ServiceBuilder::new()
        .filter(|req: GetPassengers| {
            if req.size < 2 {
                Ok(req)
            } else {
                Err("Not allowed!")
            }
        })
        .service(Client::new("https://api.instantwebtools.net"));

    let req = GetPassengers { size: 10 };
    let res = client.call(req).await.unwrap();
    res.data
        .iter()
        .for_each(|passenger| println!("{}", passenger.name.as_deref().unwrap_or("No name")));
}
