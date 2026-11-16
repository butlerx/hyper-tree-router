//! Shared helpers for the examples: a plain-text response builder and a hyper
//! accept loop that serves a `Router` on 127.0.0.1:8080.

#![allow(dead_code)] // not every example uses every helper

use http_body_util::Full;
use hyper::{
    Response, StatusCode,
    body::{Bytes, Incoming},
    header::CONTENT_TYPE,
    server::conn::http1,
};
use hyper_tree_router::Router;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

pub type Body = Full<Bytes>;

pub const ADDR: &str = "127.0.0.1:8080";

/// `200 OK` with a `text/plain` body.
pub fn text(body: impl Into<String>) -> Response<Body> {
    text_with_status(StatusCode::OK, body)
}

/// A `text/plain` response with the given status.
pub fn text_with_status(status: StatusCode, body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "text/plain")
        .body(Body::from(body.into()))
        .expect("valid response")
}

/// Serve `router` with hyper over HTTP/1, one task per connection.
pub async fn serve(
    router: Router<Incoming, Body>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(ADDR).await?;
    println!("Listening on http://{ADDR}");
    loop {
        let (stream, _) = listener.accept().await?;
        let router = router.clone();
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), router)
                .await
            {
                eprintln!("connection error: {err}");
            }
        });
    }
}
