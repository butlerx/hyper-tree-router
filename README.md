# Hyper Tree Router

[![Version](https://img.shields.io/crates/v/hyper-tree-router.svg?style=flat)](https://crates.io/crates/hyper-tree-router)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/hyper-tree-router)
[![License](https://img.shields.io/crates/l/hyper-tree-router.svg?style=flat)](https://github.com/butlerx/hyper-tree-router/blob/master/LICENSE.md)

Router for [hyper](https://hyper.rs) 1.x and [tower](https://docs.rs/tower)
that uses a prefix tree (trie) for path finding. Define routes and handlers for
a request path, then serve the router with hyper directly, wrap it in tower
middleware, or mount it inside [axum](https://docs.rs/axum).

`Router<ReqBody, ResBody>` implements both `tower::Service` and
`hyper::service::Service`. Cloning it is cheap because the route tree sits
behind an `Arc`. The only requirement on the response body is
`ResBody: From<&'static str>`, which it uses for the built-in 404/405 responses.

## Usage

Add it to your `Cargo.toml`, along with the crates the examples below import
directly (this crate doesn't re-export them):

```toml
[dependencies]
hyper-tree-router = "0.2"
http              = "1"

# Serving with hyper directly
http-body-util = "0.1"
hyper          = { version = "1", features = ["http1", "server"] }
hyper-util     = { version = "0.1", features = ["tokio"] }
tokio          = { version = "1", features = ["full"] }

# Or mounting inside axum
axum = "0.8"
```

### Defining routes

Handlers are async. Path segments use these forms:

| Pattern | Matches | Capture |
|---|---|---|
| `/user/:user_id` | exactly one segment | `params.get("user_id")` |
| `/files/*path` | one or more trailing segments, as the last segment | `params.get("path")` → `"a/b/c.txt"` |

Exact and `:param` routes take priority over `*` catch-alls, and the longest
catch-all prefix wins. `HEAD` requests fall back to the `GET` handler when a
route has no `HEAD` handler.

```rust
use http::{Request, Response};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

async fn user_handler(url_params: UrlParams, _: Request<Incoming>) -> Response<Full<Bytes>> {
    let id = url_params.get("user_id").unwrap_or_default();
    Response::new(Full::from(format!("user: {id}")))
}

let router = RouterBuilder::default()
    .route(Route::url("/user/:user_id/home").get(user_handler))
    .build();
```

### Typed params

Enable the `serde` feature to take typed params as the handler's first argument
instead of looking them up with `get`:

```toml
hyper-tree-router = { version = "0.2", features = ["serde"] }
serde             = { version = "1", features = ["derive"] }
```

```rust
use hyper_tree_router::Params;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserPath {
    user_id: u64,
}

// Struct fields are read by param name
async fn user(Params(p): Params<UserPath>, _: Request<Incoming>) -> Response<Full<Bytes>> {
    Response::new(Full::from(format!("user {}", p.user_id)))
}

// Tuples are read in path order; a single value reads the only param
async fn repo(Params((org, repo)): Params<(String, String)>, _: Request<Incoming>) -> Response<Full<Bytes>> {
    Response::new(Full::from(format!("{org}/{repo}")))
}

let router = RouterBuilder::default()
    .route(Route::url("/user/:user_id").get(user))
    .route(Route::url("/repo/:org/:repo").get(repo))
    .build();
```

Values parse from strings, so numbers, `bool`, `char`, `Option` and unit-variant
enums work. If parsing fails (say `/user/abc` for a `u64`), the handler isn't
called and the router answers `400 Bad Request` through the error handler.
`UrlParams` still works as the first argument, and you can implement
`FromUrlParams` for your own types.

### Sharing state with closures

A handler can be any `Fn(P, Request<ReqBody>) -> impl Future` that is
`Send + Sync + 'static`, where `P` is `UrlParams` or another `FromUrlParams`
type, so closures can capture shared state. Annotate the closure's first
argument so the params type can be inferred:

```rust
use std::sync::Arc;

let db = Arc::new(Db::connect().await?);
let router = RouterBuilder::default()
    .route(Route::url("/user/:user_id").get(move |params: UrlParams, _req| {
        let db = Arc::clone(&db);
        async move {
            let id = params.get("user_id").unwrap_or_default();
            Response::new(Full::from(db.user_name(&id).await))
        }
    }))
    .build();
```

### Custom error responses

Unmatched paths get a `404` and unmatched methods get a `405`. Replace the
built-in plain-text bodies with `error_handler`. On a `405` the router always
adds an `Allow` header listing the path's methods, unless your handler already
set one.

```rust
let router = RouterBuilder::default()
    .route(Route::url("/user/:user_id").get(user_handler))
    .error_handler(|status| {
        Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(Full::from(format!(r#"{{"error":{}}}"#, status.as_u16())))
            .unwrap()
    })
    .build();
```

### Serving with hyper

Pass the router to `serve_connection` once per accepted connection. See
[`examples/basic.rs`](examples/basic.rs) for a complete
server:

```sh
cargo run --example basic
curl localhost:8080/user/42/home
```

### Mounting inside axum

Use `axum::body::Body` for both body types, then nest the router under a
prefix or use it as the fallback:

```rust
use axum::body::Body;
use http::{Request, Response};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

async fn user_handler(params: UrlParams, _: Request<Body>) -> Response<Body> {
    let id = params.get("user_id").unwrap_or_default();
    Response::new(Body::from(format!("user: {id}")))
}

let tree = RouterBuilder::default()
    .route(Route::url("/user/:user_id").get(user_handler))
    .build();

let app = axum::Router::new()
    .route("/health", axum::routing::get(|| async { "ok" }))
    .nest_service("/legacy", tree); // GET /legacy/user/7 -> "user: 7"
```

## Examples

Each example serves on `127.0.0.1:8080` and lists `curl` commands to try at
the top of the file.

| Example | Shows | Run |
|---|---|---|
| [`basic`](examples/basic.rs) | Static paths, `:param` captures, several methods on one route | `cargo run --example basic` |
| [`state`](examples/state.rs) | Closure handlers sharing an `Arc<AppState>` | `cargo run --example state` |
| [`routing`](examples/routing.rs) | `*path` catch-alls, route priority, HEAD → GET fallback, 405 + `Allow` | `cargo run --example routing` |
| [`error_handler`](examples/error_handler.rs) | Custom JSON 404/405 responses | `cargo run --example error_handler` |
| [`typed_params`](examples/typed_params.rs) | `Params<T>` structs, tuples and enums; 400 on bad input | `cargo run --example typed_params --features serde` |
| [`axum`](examples/axum.rs) | Mounting in axum with `nest_service`, wrapped in a tower layer | `cargo run --example axum` |

## Migrating from 0.1

- Requires hyper 1.x. `Server::bind(..).serve(router)` is gone; serve the
  router per connection (see the example).
- `Route`, `RouterBuilder` and `Router` are now generic over
  `<ReqBody, ResBody>`. Handlers take `Request<ReqBody>` and return
  `Response<ResBody>` rather than using `hyper::Body`.
- `Router` is now the request service itself. `Router.routes` is no longer
  public.
- Captured parameters are keyed by bare name: `params.get("user_id")`, not
  `captures.get(":user_id")`. The `captures` `HashMap` is now private; use
  `get`, `iter`, `len` and `is_empty`. `UrlParams::new()` and
  `RouterBuilder::new()` are gone; use `::default()`.
- A handler's first argument can be any `FromUrlParams` type, such as
  `Params<T>` with the `serde` feature. Closures need their first argument
  annotated (`|p: UrlParams, req| ..`).
- Handlers are async (`async fn`, or a closure returning a future) instead of
  plain `fn` pointers, and closures can capture state. `Handler` is now a
  type-erased `Arc<dyn Fn ..>`.
