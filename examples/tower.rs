use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use stream_flatten_iters::TryStreamExt as _;
use tower::ServiceBuilder;
use tower_jsonapi_client::{Client, Request, RequestData};

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

#[tokio::main]
pub async fn main() {
    use tower::ServiceExt;
    env_logger::init();
    let client = ServiceBuilder::new()
        .rate_limit(5, std::time::Duration::from_secs(1))
        .service(Client::new("https://api.instantwebtools.net"));

    let reqs = futures::stream::iter([1, 2, 3]).map(|x| GetPassengers {
        page: Some(x),
        size: 10,
    });

    client
        .call_all(reqs)
        .map(|x| x.map(|y| y.data))
        .try_flatten_iters()
        .try_for_each(|res| async {
            println!("{}", res.name.unwrap_or_else(|| String::from("No name")));
            Ok(())
        })
        .await
        .unwrap();
}
