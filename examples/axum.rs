//! Mounting the tree router inside an axum app, wrapped in a tower layer.
//!
//! ```sh
//! cargo run --example axum
//! curl localhost:8080/health             # axum route: ok
//! curl -i localhost:8080/legacy/user/7   # tree router: user 7, plus x-served-by header
//! curl -i localhost:8080/legacy/missing  # tree router's 404
//! ```

use axum::body::Body;
use http::{HeaderValue, Request, Response};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};
use tower::ServiceBuilder;

async fn user(params: UrlParams, _: Request<Body>) -> Response<Body> {
    Response::new(Body::from(format!(
        "user {}",
        params.get("user_id").unwrap_or_default()
    )))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Inside axum both body types are `axum::body::Body`.
    let tree = RouterBuilder::default()
        .route(Route::url("/user/:user_id").get(user))
        .build();

    // The router is a plain tower `Service`, so tower layers wrap it directly.
    let tree = ServiceBuilder::new()
        .map_response(|mut res: Response<Body>| {
            res.headers_mut()
                .insert("x-served-by", HeaderValue::from_static("hyper-tree-router"));
            res
        })
        .service(tree);

    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .nest_service("/legacy", tree);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on http://127.0.0.1:8080");
    axum::serve(listener, app).await?;
    Ok(())
}
