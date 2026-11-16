//! Basic routing served directly by hyper: static paths, `:param` captures and
//! several methods on one route.
//!
//! ```sh
//! cargo run --example basic
//! curl localhost:8080/
//! curl localhost:8080/user/42/home
//! curl -X POST localhost:8080/user/42/home
//! ```

mod common;

use common::{Body, serve, text};
use hyper::{Request, Response, body::Incoming};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

async fn hello(_: UrlParams, _: Request<Incoming>) -> Response<Body> {
    text("Hello World")
}

async fn show_user(params: UrlParams, _: Request<Incoming>) -> Response<Body> {
    text(format!(
        "user: {}",
        params.get("user_id").unwrap_or("unknown")
    ))
}

async fn update_user(params: UrlParams, _: Request<Incoming>) -> Response<Body> {
    text(format!(
        "updated user {}",
        params.get("user_id").unwrap_or("unknown")
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = RouterBuilder::default()
        .route(Route::url("/").get(hello))
        .route(
            Route::url("/user/:user_id/home")
                .get(show_user)
                .post(update_user),
        )
        .build();
    serve(router).await
}
