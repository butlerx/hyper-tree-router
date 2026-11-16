//! Catch-all segments, route priority and the HEAD -> GET fallback.
//!
//! ```sh
//! cargo run --example routing
//! curl localhost:8080/files/docs/guide.md   # catch-all: docs/guide.md
//! curl localhost:8080/files/readme          # exact route wins
//! curl localhost:8080/files/notes/raw       # :param route wins: raw notes
//! curl -I localhost:8080/files/readme       # HEAD served by the GET handler
//! curl -i -X DELETE localhost:8080/files/readme  # 405 with `allow: GET, HEAD`
//! ```

mod common;

use common::{Body, serve, text};
use hyper::{Request, Response, body::Incoming};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

async fn any_file(params: UrlParams, _: Request<Incoming>) -> Response<Body> {
    // `*path` captures the rest of the path, joined with `/`.
    text(format!(
        "catch-all: {}",
        params.get("path").unwrap_or_default()
    ))
}

async fn readme(_: UrlParams, _: Request<Incoming>) -> Response<Body> {
    text("exact route: readme")
}

async fn raw(params: UrlParams, _: Request<Incoming>) -> Response<Body> {
    text(format!("raw {}", params.get("name").unwrap_or_default()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = RouterBuilder::default()
        .route(Route::url("/files/*path").get(any_file))
        .route(Route::url("/files/readme").get(readme))
        .route(Route::url("/files/:name/raw").get(raw))
        .build();
    serve(router).await
}
