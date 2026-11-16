//! Closure handlers that capture shared state.
//!
//! ```sh
//! cargo run --example state
//! curl localhost:8080/hits      # 1
//! curl localhost:8080/hits      # 2
//! curl localhost:8080/greet/ada # "Hi ada, from demo-app"
//! ```

mod common;

use common::{serve, text};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

struct AppState {
    name: String,
    hits: AtomicU64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(AppState {
        name: "demo-app".into(),
        hits: AtomicU64::new(0),
    });

    let hits = Arc::clone(&state);
    let greet = Arc::clone(&state);
    let router = RouterBuilder::default()
        // Annotate the first argument so the params type can be inferred.
        .route(Route::url("/hits").get(move |_: UrlParams, _| {
            let state = Arc::clone(&hits);
            async move {
                text(format!(
                    "{}",
                    state.hits.fetch_add(1, Ordering::Relaxed) + 1
                ))
            }
        }))
        .route(Route::url("/greet/:name").get(move |params: UrlParams, _| {
            let state = Arc::clone(&greet);
            async move {
                text(format!(
                    "Hi {}, from {}",
                    params.get("name").unwrap_or("stranger"),
                    state.name
                ))
            }
        }))
        .build();
    serve(router).await
}
