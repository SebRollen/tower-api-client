use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tower_jsonapi_client::{EmptyResponse, Request, RequestData};

pub mod matchers;

pub struct EmptyHello;

impl Request for EmptyHello {
    type Data = ();
    type Response = EmptyResponse;

    fn endpoint(&self) -> Cow<str> {
        "/hello".into()
    }
}

#[derive(Serialize)]
pub struct QueryHello {
    pub name: String,
}

#[derive(Serialize)]
pub struct JsonHello {
    pub name: String,
}

#[derive(Serialize)]
pub struct FormHello {
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct NameGreeting {
    pub message: String,
}

impl Request for QueryHello {
    type Data = Self;
    type Response = NameGreeting;

    fn endpoint(&self) -> Cow<str> {
        "/hello".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Query(self)
    }
}

impl Request for JsonHello {
    type Data = Self;
    type Response = NameGreeting;

    fn endpoint(&self) -> Cow<str> {
        "/hello".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Json(self)
    }
}

impl Request for FormHello {
    type Data = Self;
    type Response = NameGreeting;

    fn endpoint(&self) -> Cow<str> {
        "/hello".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Form(self)
    }
}
