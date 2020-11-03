use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::state::{FromState, State};
use gotham_derive::NewMiddleware;
use http::header::HeaderMap;
use pin_project::pin_project;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use mightybadger::payload::RequestInfo;

#[derive(Clone, NewMiddleware)]
pub struct HoneybadgerMiddleware;

#[pin_project]
struct WithRequestContext<F> {
    #[pin]
    inner: F,
    context: RequestInfo,
}

impl<F> WithRequestContext<F> {
    fn new(inner: F, context: RequestInfo) -> Self {
        Self { inner, context }
    }
}

impl<F: Future> Future for WithRequestContext<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let inner = this.inner;
        mightybadger::context::with(&this.context, || inner.poll(ctx))
    }
}

impl Middleware for HoneybadgerMiddleware {
    fn call<Chain>(self, state: State, chain: Chain) -> Pin<Box<HandlerFuture>>
    where
        Chain: FnOnce(State) -> Pin<Box<HandlerFuture>>,
    {
        let request_info = {
            let mut cgi_data = HashMap::new();
            let headers = HeaderMap::borrow_from(&state);
            for (name, value) in headers.iter() {
                let name = "HTTP_"
                    .chars()
                    .chain(name.as_str().chars())
                    .map(|ch| {
                        if ch == '-' {
                            '_'
                        } else {
                            ch.to_ascii_uppercase()
                        }
                    })
                    .collect::<String>();
                cgi_data.insert(name, String::from_utf8_lossy(value.as_bytes()).into_owned());
            }
            RequestInfo {
                cgi_data: cgi_data,
                ..Default::default()
            }
        };
        let f = mightybadger::context::with(&request_info, || chain(state));
        let f = WithRequestContext::new(f, request_info);
        Box::pin(f)
    }
}
