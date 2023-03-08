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
    pub struct PaginationStream<Svc, T, R, Q> {
        state: State<T>,
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

impl<Svc: Service<R>, T, R> PaginationStream<Svc, T, R, FuturesOrdered<Svc::Future>>
where
    T: Clone,
    Svc: Service<R, Response = R::Response>,
    R: PaginatedRequest<PaginationData = T>,
{
    pub(crate) fn new(svc: Svc, request: R) -> Self {
        Self {
            state: State::Start(None),
            svc,
            queue: FuturesOrdered::new(),
            request,
        }
    }
}

impl<Svc, T, R, Q> Stream for PaginationStream<Svc, T, R, Q>
where
    T: Clone,
    Svc: Service<R, Response = R::Response>,
    R: PaginatedRequest<PaginationData = T>,
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

                this.queue.push(this.svc.call(this.request.clone()));
                cx.waker().wake_by_ref();

                Poll::Pending
            }
        } else {
            Poll::Pending
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
