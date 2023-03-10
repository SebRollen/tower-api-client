//! Constructs for wrapping a paginated API.
use crate::request::Request;
use futures::{ready, Stream};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;

pub trait PaginatedRequest: Request + Clone {
    type PaginationData;
    fn get_page(&self) -> Option<Self::PaginationData>;
    fn next_page(
        &self,
        prev_page: Option<&Self::PaginationData>,
        response: &Self::Response,
    ) -> Option<Self::PaginationData>;
    fn update_request(&mut self, page: &Self::PaginationData);
}

pin_project! {
    pub struct PaginationStream<Svc: Service<R>, T, R> {
        state: State<T>,
        svc: Svc,
        future: Option<Pin<Box<Svc::Future>>>,
        request: R,
    }
}

impl<Svc: Service<R>, T, R: PaginatedRequest<PaginationData = T>> PaginationStream<Svc, T, R> {
    pub(crate) fn new(svc: Svc, request: R) -> Self {
        let page = request.get_page();
        Self {
            state: State::Start(page),
            svc,
            future: None,
            request,
        }
    }
}

impl<Svc, T, R> Stream for PaginationStream<Svc, T, R>
where
    T: Clone + std::fmt::Debug,
    Svc: Service<R, Response = R::Response>,
    Svc::Future: Unpin,
    R: PaginatedRequest<PaginationData = T>,
{
    type Item = Result<Svc::Response, Svc::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut page = match this.state {
            State::Start(None) => None,
            State::Start(Some(state)) | State::Next(state) => Some(state.clone()),
            State::End => {
                return Poll::Ready(None);
            }
        };

        loop {
            match this.future {
                Some(fut) => {
                    let response = ready!(fut.as_mut().poll(cx));
                    // The future has completed, so we replace it with none to make sure it doesn't
                    // get polled again
                    *this.future = None;
                    let response = response?;
                    page = this.request.next_page(page.as_ref(), &response);
                    if let Some(page) = page {
                        *this.state = State::Next(page)
                    } else {
                        *this.state = State::End
                    }

                    return Poll::Ready(Some(Ok(response)));
                }
                None => {
                    if let Err(e) = ready!(this.svc.poll_ready(cx)) {
                        return Poll::Ready(Some(Err(e)));
                    }

                    if let Some(page) = page.as_ref() {
                        this.request.update_request(page);
                    }

                    *this.future = Some(Box::pin(this.svc.call(this.request.clone())));
                }
            }
        }
    }
}

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
