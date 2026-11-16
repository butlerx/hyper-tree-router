//! Custom 404/405/400 responses. The router still adds the `Allow` header to 405s.
//!
//! ```sh
//! cargo run --example error_handler
//! curl -i localhost:8080/missing          # 404 {"error":"Not Found"}
//! curl -i -X POST localhost:8080/status   # 405 {"error":"Method Not Allowed"} + allow: GET, HEAD
//! ```

mod common;

use common::{Body, serve, text};
use hyper::{Request, Response, body::Incoming, header::CONTENT_TYPE};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

async fn status(_: UrlParams, _: Request<Incoming>) -> Response<Body> {
    text("ok")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = RouterBuilder::default()
        .route(Route::url("/status").get(status))
        .error_handler(|status| {
            let reason = status.canonical_reason().unwrap_or("Error");
            Response::builder()
                .status(status)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(format!(r#"{{"error":"{reason}"}}"#)))
                .expect("valid response")
        })
        .build();
    serve(router).await
}
