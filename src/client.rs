use crate::error::{Error, Result};
use crate::pagination::{PaginatedRequest, PaginationStream};
use crate::request::{Request, RequestData};
use base64::{engine::general_purpose::STANDARD, Engine};
use futures::{prelude::*, stream::FuturesOrdered};
use hyper::{
    body::{to_bytes, Body},
    client::HttpConnector,
    http::request::Builder,
    Client as HyperClient,
};
use hyper_tls::HttpsConnector;
use log::debug;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;

#[derive(Clone)]
enum Authorization {
    Bearer(String),
    Basic(String, Option<String>),
    Query(HashMap<String, String>),
    Header(HeaderMap<HeaderValue>),
}

/// The main client used for making requests.
///
/// `Client` stores an async Reqwest client as well as the associated
/// base url and possible authorization details for the REST server.
#[derive(Clone)]
pub struct Client {
    inner: HyperClient<HttpsConnector<HttpConnector>, Body>,
    base_url: String,
    auth: Option<Authorization>,
}

impl<R: Request + 'static> Service<R> for Client {
    type Response = R::Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>>>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: R) -> Self::Future {
        let this = self.clone();
        Box::pin(async move { this.send(request).await })
    }
}

impl Client {
    /// Create a new `Client`.
    pub fn new<S: ToString>(base_url: S) -> Self {
        let connector = HttpsConnector::new();
        let client = HyperClient::builder().build(connector);

        Self::from_hyper(client, base_url)
    }

    /// Create a new `Client` from an existing Reqwest Client.
    pub fn from_hyper<S: ToString>(
        inner: HyperClient<HttpsConnector<HttpConnector>>,
        base_url: S,
    ) -> Self {
        Self {
            inner,
            base_url: base_url.to_string(),
            auth: None,
        }
    }

    /// Enable bearer authentication for the client
    pub fn bearer_auth<S: ToString>(mut self, token: S) -> Self {
        self.auth = Some(Authorization::Bearer(token.to_string()));
        self
    }

    /// Enable basic authentication for the client
    pub fn basic_auth<T: Into<Option<S>>, S: ToString>(mut self, user: S, pass: T) -> Self {
        self.auth = Some(Authorization::Basic(
            user.to_string(),
            pass.into().map(|x| x.to_string()),
        ));
        self
    }

    /// Enable query authentication for the client
    pub fn query_auth<S: ToString>(mut self, pairs: Vec<(S, S)>) -> Self {
        let pairs = pairs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.auth = Some(Authorization::Query(pairs));
        self
    }

    /// Enable custom header authentication for the client
    pub fn header_auth<S: ToString>(mut self, pairs: Vec<(S, S)>) -> Self {
        let mut map = HeaderMap::new();
        for (k, v) in pairs {
            let k = k.to_string();
            let v = v.to_string();
            let mut header_value = HeaderValue::from_str(&v).expect("Failed to create HeaderValue");
            header_value.set_sensitive(true);
            map.insert(
                HeaderName::try_from(&k).expect("Failed to create HeaderName"),
                header_value,
            );
        }
        self.auth = Some(Authorization::Header(map));
        self
    }

    fn send_raw<R>(&self, req: hyper::Request<Body>) -> impl std::future::Future<Output = Result<R>>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        debug!("Sending request: {:?}", req);
        self.inner
            .request(req)
            .map_err(From::from)
            .and_then(|mut res| async move {
                let status = res.status();
                let body = to_bytes(res.body_mut()).await?;
                if status.is_success() {
                    serde_json::from_slice(&body).map_err(From::from)
                } else if status.is_client_error() {
                    Err(Error::ClientError(status, String::from_utf8(body.into())?))
                } else {
                    Err(Error::ServerError(status, String::from_utf8(body.into())?))
                }
            })
    }

    fn format_request<R: Request>(&self, request: &R) -> Result<hyper::Request<Body>> {
        let endpoint = request.endpoint();
        let endpoint = endpoint.trim_matches('/');
        let url = format!("{}/{}", self.base_url, endpoint);

        let mut req = Builder::new().uri(&url).method(R::METHOD);
        req.headers_mut().replace(&mut request.headers());

        req = match &self.auth {
            None => req,
            Some(Authorization::Bearer(token)) => {
                req.header("authorization", format!("Bearer {}", token))
            }
            Some(Authorization::Basic(user, pass)) => {
                let creds = format!("{}:{}", user, pass.as_deref().unwrap_or(""));
                let encoded = STANDARD.encode(creds);
                req.header("authorization", format!("Basic {}", encoded))
            }
            Some(Authorization::Query(pairs)) => {
                let query = serde_qs::to_string(pairs)?;
                let url = if url.contains('?') {
                    format!("{}&{}", url, query)
                } else {
                    format!("{}?{}", url, query)
                };
                req.uri(url)
            }
            Some(Authorization::Header(pairs)) => {
                for (k, v) in pairs {
                    req = req.header(k, v);
                }
                req
            }
        };

        let body = match request.data() {
            RequestData::Empty => Body::empty(),
            RequestData::Form(data) => {
                req = req
                    .header("content-type", "application/x-www-form-urlencoded")
                    .uri(url);
                let body = serde_urlencoded::to_string(data)?;
                Body::from(body)
            }
            RequestData::Json(data) => {
                req = req.header("content-type", "application/json").uri(url);
                let bytes = serde_json::to_vec(&data)?;
                Body::from(bytes)
            }
            RequestData::Query(data) => {
                let query = serde_qs::to_string(data)?;
                let url = if url.contains('?') {
                    format!("{}&{}", url, query)
                } else {
                    format!("{}?{}", url, query)
                };
                req = req.uri(url);
                Body::empty()
            }
        };

        req.body(body).map_err(From::from)
    }

    /// Send a single `Request`
    pub async fn send<R: Request>(&self, request: R) -> Result<R::Response> {
        let req = self.format_request(&request)?;
        self.send_raw(req).await
    }
}

pub trait ServiceExt<R: PaginatedRequest<PaginationData = T>, T>: Service<R> {
    fn paginate(
        self,
        request: R,
    ) -> PaginationStream<Self, T, R, FuturesOrdered<<Self as Service<R>>::Future>>
    where
        T: Clone,
        R: Request<Response = <Self as Service<R>>::Response>,
        R: PaginatedRequest,
        Self: Sized,
    {
        PaginationStream::new(self, request)
    }
}

impl<P, T: ?Sized, Request> ServiceExt<Request, P> for T
where
    T: Service<Request>,
    Request: PaginatedRequest<PaginationData = P>,
{
}
