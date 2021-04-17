use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::create_response;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use http::{Response, StatusCode};
use hyper::Body;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;

fn router() -> Router {
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(mightybadger_gotham::HoneybadgerMiddleware)
            .build(),
    );
    build_router(chain, pipelines, |route| {
        route.get("/").to(index);
        route.get("/ping").to(ping);
        route.get("/error").to(error);
        route.get("/error_wait").to(error_wait);
    })
}

fn index(state: State) -> (State, Response<Body>) {
    let bytes = "Hello, world!".to_string().into_bytes();
    let res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, bytes);

    (state, res)
}

fn ping(state: State) -> (State, Response<Body>) {
    let bytes = "pong".to_string().into_bytes();
    let res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, bytes);

    (state, res)
}

fn error(_state: State) -> (State, Response<Body>) {
    panic!("/error is requested");
}

fn error_wait(_state: State) -> Pin<Box<HandlerFuture>> {
    Box::pin(async {
        sleep(Duration::from_millis(1000)).await;
        panic!("/error_wait is requested");
    })
}

fn main() {
    mightybadger::setup();
    let addr = "127.0.0.1:7878";
    gotham::start(addr, router())
}
