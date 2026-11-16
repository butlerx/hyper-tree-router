//! Typed URL params with the `serde` feature.
//!
//! ```sh
//! cargo run --example typed_params --features serde
//! curl localhost:8080/user/42                # struct, by name: user 42
//! curl localhost:8080/repo/acme/widget       # tuple, in path order: acme/widget
//! curl localhost:8080/repo/acme/widget/pulls # enum: acme/widget pulls
//! curl -i localhost:8080/user/abc            # 400: not a u64, handler not called
//! ```

mod common;

use common::{Body, serve, text};
use hyper::{Request, Response, body::Incoming};
use hyper_tree_router::{Params, Route, RouterBuilder};
use serde::Deserialize;

#[derive(Deserialize)]
struct UserPath {
    user_id: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Tab {
    Issues,
    Pulls,
}

async fn user(Params(path): Params<UserPath>, _: Request<Incoming>) -> Response<Body> {
    text(format!("user {}", path.user_id))
}

async fn repo(
    Params((org, repo)): Params<(String, String)>,
    _: Request<Incoming>,
) -> Response<Body> {
    text(format!("{org}/{repo}"))
}

async fn repo_tab(
    Params((org, repo, tab)): Params<(String, String, Tab)>,
    _: Request<Incoming>,
) -> Response<Body> {
    text(format!(
        "{org}/{repo} {}",
        format!("{tab:?}").to_lowercase()
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = RouterBuilder::default()
        .route(Route::url("/user/:user_id").get(user))
        .route(Route::url("/repo/:org/:repo").get(repo))
        .route(Route::url("/repo/:org/:repo/:tab").get(repo_tab))
        .build();
    serve(router).await
}
