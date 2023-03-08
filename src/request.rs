use hyper::{header::HeaderMap, Method};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Cow;

/// Additional data to be sent along with the request.
pub enum RequestData<T> {
    /// No additional data.
    Empty,
    /// HTTP form data.
    Form(T),
    /// JSON data.
    Json(T),
    /// Query data.
    Query(T),
}

impl<T> Default for RequestData<T> {
    fn default() -> Self {
        RequestData::Empty
    }
}

/// The base-trait for requests sent by the client. The trait specifies the full life-cycle of the
/// request, including the endpoint, headers, data, method and eventual response.
pub trait Request {
    /// The type of additional data sent with the request. Usually, this will be `()` or `Self`.
    type Data: Serialize;
    /// The type of the response from the server.
    type Response: for<'de> Deserialize<'de> + Unpin;

    /// The HTTP method for the request.
    const METHOD: Method = Method::GET;

    /// The endpoint to which the request will be sent. The base url is set in the client, and the
    /// endpoint method returns the specific resource endpoint.
    fn endpoint(&self) -> Cow<str>;

    /// Any additional headers that should be sent with the request. Note that common headers such
    /// as authorization headers should be set on the client directly.
    fn headers(&self) -> HeaderMap {
        Default::default()
    }

    /// The formatted request data.
    fn data(&self) -> RequestData<&Self::Data> {
        Default::default()
    }
}

#[derive(Debug)]
/// Struct symbolizing an empty response from the server.
pub struct EmptyResponse;
impl<'de> Deserialize<'de> for EmptyResponse {
    fn deserialize<D>(_deserializer: D) -> Result<EmptyResponse, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(EmptyResponse {})
    }
}
