//! Constructs for wrapping a paginated API.
use crate::request::Request;
use futures::stream::FuturesOrdered;
use futures::{ready, Stream};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;

pub trait PaginatedRequest: Request + Clone {
    type PaginationData;
    fn next_page(
        &self,
        prev_page: Option<&Self::PaginationData>,
        response: &Self::Response,
    ) -> Option<Self::PaginationData>;
    fn update_request(&mut self, page: &Self::PaginationData);
}

pin_project! {
    pub struct PaginationStream<Svc, R, Q> {
        state: State<i32>,
        svc: Svc,
        queue: Q,
        request: R,
    }
}

pub(crate) trait Drive<F: Future> {
    fn is_empty(&self) -> bool;

    fn push(&mut self, future: F);

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Option<F::Output>>;
}

impl<F: Future> Drive<F> for FuturesOrdered<F> {
    fn is_empty(&self) -> bool {
        FuturesOrdered::is_empty(self)
    }

    fn push(&mut self, future: F) {
        FuturesOrdered::push_back(self, future)
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Option<F::Output>> {
        Stream::poll_next(Pin::new(self), cx)
    }
}

impl<Svc: Service<R>, R> PaginationStream<Svc, R, FuturesOrdered<Svc::Future>> {
    pub(crate) fn new(svc: Svc, request: R) -> Self {
        Self {
            state: State::Start(None),
            svc,
            queue: FuturesOrdered::new(),
            request,
        }
    }
}

impl<Svc, R, Q> Stream for PaginationStream<Svc, R, Q>
where
    Svc: Service<R, Response = R::Response>,
    R: PaginatedRequest<PaginationData = i32> + std::fmt::Debug,
    Q: Drive<Svc::Future>,
{
    type Item = Result<Svc::Response, Svc::Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut page = match this.state {
            State::Start(None) => None,
            State::Start(Some(state)) | State::Next(state) => Some(state.clone()),
            State::End => {
                if this.queue.is_empty() {
                    return Poll::Ready(None);
                } else {
                    return Poll::Pending;
                }
            }
        };

        if let Poll::Ready(r) = this.queue.poll(cx) {
            if let Some(rsp) = r.transpose().map_err(Into::into)? {
                page = this.request.next_page(page.as_ref(), &rsp);
                if let Some(page) = page {
                    *this.state = State::Next(page)
                } else {
                    *this.state = State::End
                }

                return Poll::Ready(Some(Ok(rsp)));
            } else {
                if let Err(e) = ready!(this.svc.poll_ready(cx)) {
                    return Poll::Ready(Some(Err(e)));
                }

                if let Some(page) = page.as_ref() {
                    this.request.update_request(page);
                }

                println!("{:?}", this.request);

                this.queue.push(this.svc.call(this.request.clone()));
                cx.waker().wake_by_ref();

                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}

// #[derive(Clone)]
// pub struct Paginator {
//     client: Client,
// }
//
// impl Paginator {
//     pub fn new(client: Client) -> Self {
//         Self { client }
//     }
// }
//
// impl<R: 'static + Request> Service<R> for Paginator {
//     type Response = Pin<Box<dyn Stream<Item = Result<R::Response, Error>>>>;
//     type Error = Error;
//     type Future = Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Error>>>>;
//
//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Error>> {
//         self.client.poll_ready(cx)
//     }
//
//     fn call(&mut self, req: R) -> Self::Future {
//         let this = self.clone();
//         let hyper_request = self.client.format_request(&req).unwrap();
//         let (parts, body) = hyper_request.into_parts();
//         let (method, uri, version, headers) =
//             (parts.method, parts.uri, parts.version, parts.headers);
//         Box::pin(futures::future::ok(
//             futures::stream::try_unfold(
//                 (this.clone(), State::Start(None)),
//                 move |(paginator, state): (Paginator, State<i32>)| {
//                     let this = this.clone();
//                     let method = method.clone();
//                     let mut uri = uri.clone();
//                     let mut headers = headers.clone();
//                     let body = body.clone();
//                     async move {
//                         let page = match state {
//                             State::Start(None) => None,
//                             State::Start(Some(ref page)) | State::Next(ref page) => Some(page),
//                             State::End => {
//                                 return Ok(None);
//                             }
//                         };
//                         if let Some(page) = page {
//                             // Switch to Url, because working with hyper's uri sucks
//                             let url = url::Url::parse(&uri.to_string()).unwrap();
//                             let mut parts = uri.into_parts();
//                             let unchanged_queries: Vec<(_, _)> = url
//                                 .query_pairs()
//                                 .filter(|(k, _)| !(k.as_ref() == "page"))
//                                 .collect();
//                             let mut temp_url = url.clone();
//                             temp_url.set_query(None);
//                             for (key, val) in unchanged_queries {
//                                 temp_url.query_pairs_mut().append_pair(&key, &val);
//                             }
//                             let page = page.to_string();
//                             temp_url.query_pairs_mut().append_pair("page", &page);
//                             let path = temp_url.path();
//                             let query = temp_url.query().unwrap();
//                             use std::str::FromStr;
//                             parts.path_and_query = Some(
//                                 hyper::http::uri::PathAndQuery::from_str(
//                                     format!("{}?{}", path, query).as_str(),
//                                 )
//                                 .unwrap(),
//                             );
//                             uri = hyper::Uri::from_parts(parts).unwrap();
//                         }
//                         let mut req = HyperRequest::builder()
//                             .method(method.clone())
//                             .uri(uri.clone())
//                             .version(version.clone());
//                         req.headers_mut().replace(&mut headers);
//                         let req = req.body(body).unwrap();
//                         let response = this.clone().client.call(req).await?;
//                         let state = match page {
//                             None => State::Start(Some(1)),
//                             Some(page) if page < &2 => State::Next(page + 1),
//                             Some(_) => State::End,
//                         };
//                         Ok(Some((response, (paginator, state))))
//                     }
//                 },
//             )
//             .boxed_local(),
//         ))
//     }
// }

// /// Trait for updating an HTTP request with pagination data.
// pub trait RequestModifier {
//     /// Modify the request with updated pagination data.
//     fn modify_request(&self, request: &mut RawRequest) -> Result<()>;
// }
//
// /// Base trait for paginators. Paginators can use the previous pagination state
// /// and the response from the previous request to create a new pagination state.
// pub trait Paginator<T, U> {
//     /// The associated modifier that modifies the request with new pagination data.
//     type Modifier: RequestModifier;
//
//     /// Constructs an associated modifier using pagination data.
//     fn modifier(&self, data: U) -> Self::Modifier;
//     /// Method for returning the next pagination state given the previous pagination data and the results from the previous request.
//     fn next(&self, prev: Option<&U>, res: &T) -> State<U>;
// }
//
// /// Trait for any request that requires pagination.
// pub trait PaginatedRequest: Request {
//     /// Associated data that can be used for pagination.
//     type Data: Clone;
//
//     /// The paginator used for the request.
//     type Paginator: Paginator<Self::Response, <Self as PaginatedRequest>::Data>;
//
//     /// Return the associated paginator.
//     fn paginator(&self) -> Self::Paginator;
//
//     /// Specify the initial page to start pagination from. Defaults to `None`, which means
//     /// pagination will begin from whatever page the API defines as the initial page.
//     fn initial_page(&self) -> Option<<Self as PaginatedRequest>::Data> {
//         None
//     }
// }
//
#[derive(Clone, Debug)]
/// The current pagination state.
pub enum State<T> {
    /// State associated with the initial request.
    Start(Option<T>),
    /// State associated with continuing pagination.
    Next(T),
    /// State denoting that the last page has been reached.
    End,
}

impl<T> Default for State<T> {
    fn default() -> State<T> {
        State::Start(None)
    }
}

// pub mod query {
//     //! Constructs for working with APIs that implement paging through one or more query parameters.
//     use super::*;
//     #[derive(Debug, Clone)]
//     /// A modifier that updates the query portion of a request's URL. This modifier updates the
//     /// query keys using the values inside the data HashMap, overwriting any existing fields and
//     /// appending any non-existing fields.
//     pub struct QueryModifier {
//         pub data: HashMap<String, String>,
//     }
//
//     impl RequestModifier for QueryModifier {
//         fn modify_request(&self, request: &mut RawRequest) -> Result<()> {
//             let url = request.url_mut();
//             let unchanged_queries: Vec<(_, _)> = url
//                 .query_pairs()
//                 .filter(|(k, _)| !self.data.contains_key(k.as_ref()))
//                 .collect();
//             let mut temp_url = url.clone();
//             temp_url.set_query(None);
//             for (key, val) in unchanged_queries {
//                 temp_url.query_pairs_mut().append_pair(&key, &val);
//             }
//             for (key, val) in self.data.iter() {
//                 temp_url.query_pairs_mut().append_pair(key, val);
//             }
//             url.set_query(temp_url.query());
//             Ok(())
//         }
//     }
//
//     /// A paginator that implements pagination through one or more query parameters.
//     pub struct QueryPaginator<T, U> {
//         #[allow(clippy::type_complexity)]
//         f: Box<dyn 'static + Send + Sync + Fn(Option<&U>, &T) -> Option<U>>,
//     }
//
//     impl<T, U> QueryPaginator<T, U> {
//         #[allow(clippy::type_complexity)]
//         pub fn new<F: 'static + Send + Sync + Fn(Option<&U>, &T) -> Option<U>>(f: F) -> Self {
//             Self { f: Box::new(f) }
//         }
//     }
//
//     impl<T, U> Paginator<T, U> for QueryPaginator<T, U>
//     where
//         U: Into<QueryModifier>,
//     {
//         type Modifier = QueryModifier;
//
//         fn modifier(&self, data: U) -> QueryModifier {
//             data.into()
//         }
//
//         fn next(&self, prev: Option<&U>, res: &T) -> State<U> {
//             let queries = (self.f)(prev, res);
//             match queries {
//                 Some(queries) => State::Next(queries),
//                 None => State::End,
//             }
//         }
//     }
// }
//
// pub mod path {
//     //! Constructs for working with APIs that implement paging through one or more path parameters.
//     use super::*;
//     #[derive(Debug, Clone)]
//
//     /// A modifier that updates the path portion of a request's URL. This modifier holds a HashMap
//     /// that maps the position of a path parameter to its updated value.
//     pub struct PathModifier {
//         pub data: HashMap<usize, String>,
//     }
//
//     impl RequestModifier for PathModifier {
//         fn modify_request(&self, request: &mut RawRequest) -> Result<()> {
//             let url = request.url_mut();
//             let temp_url = url.clone();
//             let mut new_segments: Vec<&str> = temp_url
//                 .path_segments()
//                 .ok_or_else(|| Error::Pagination {
//                     msg: "URL cannot be a base".to_string(),
//                 })?
//                 .enumerate()
//                 .map(|(i, x)| self.data.get(&i).map(|val| val.as_str()).unwrap_or(x))
//                 .collect();
//             let len = new_segments.len();
//             // Append any additional path segments not present in original path
//             new_segments.extend(self.data.iter().filter_map(|(i, x)| {
//                 if *i >= len {
//                     Some(x.as_str())
//                 } else {
//                     None
//                 }
//             }));
//             let mut path_segments = url.path_segments_mut().map_err(|_| Error::Pagination {
//                 msg: "URL cannot be a base".to_string(),
//             })?;
//             path_segments.clear();
//             path_segments.extend(new_segments.iter());
//             Ok(())
//         }
//     }
//
//     /// A paginator that implements pagination through one or more path parameters. The closure inside
//     /// the paginator should return the path segment number and the new path segment, e.g. (2, "foo")
//     /// represents changing the third path segment to "foo"
//     pub struct PathPaginator<T, U> {
//         #[allow(clippy::type_complexity)]
//         f: Box<dyn 'static + Send + Sync + Fn(Option<&U>, &T) -> Option<U>>,
//     }
//
//     impl<T, U> PathPaginator<T, U> {
//         pub fn new<F: 'static + Send + Sync + Fn(Option<&U>, &T) -> Option<U>>(f: F) -> Self {
//             Self { f: Box::new(f) }
//         }
//     }
//
//     impl<T, U> Paginator<T, U> for PathPaginator<T, U>
//     where
//         U: Into<PathModifier>,
//     {
//         type Modifier = PathModifier;
//         fn modifier(&self, data: U) -> Self::Modifier {
//             data.into()
//         }
//         fn next(&self, prev: Option<&U>, res: &T) -> State<U> {
//             let path = (self.f)(prev, res);
//             match path {
//                 Some(path) => State::Next(path),
//                 None => State::End,
//             }
//         }
//     }
// }
//
// //enum PaginationFn<T, U> {
// //    NoResult(Box<dyn Send + Sync + Fn(Option<&U>) -> Option<U>>),
// //    ResultNeeded(Box<dyn Send + Sync + Fn(Option<&U>, &T) -> Option<U>>),
// //}
// //
// //impl<'u, T, U: 'u> From<fn(Option<&'u U>) -> Option<U>> for PaginationFn<T, U> {
// //    fn from(x: fn(Option<&'u U>) -> Option<U>) -> PaginationFn<T, U> {
// //        PaginationFn::NoResult(Box::new(x))
// //    }
// //}
// //
// //impl<T: 'static, U: 'static> From<fn(Option<&U>, &T) -> Option<U>> for PaginationFn<T, U> {
// //    fn from(x: fn(Option<&U>, &T) -> Option<U>) -> PaginationFn<T, U> {
// //        PaginationFn::ResultNeeded(Box::new(x))
// //    }
// //}
// //
// //fn test(x: Option<&i32>) -> Option<i32> {
// //    x.cloned()
// //}
// //
// //fn new_pagination<T>() -> PaginationFn<T, i32> {
// //    test.into()
// //}
