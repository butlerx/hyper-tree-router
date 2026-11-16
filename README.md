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

URL parameters use the form `:param_name`. Captured values are in
`url_params.captures`, keyed by the parameter name including the leading `:`.

```rust
use http::{Request, Response};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

fn user_handler(url_params: UrlParams, _: Request<Incoming>) -> Response<Full<Bytes>> {
    let id = url_params.captures.get(":user_id").cloned().unwrap_or_default();
    Response::new(Full::from(format!("user: {id}")))
}

let router = RouterBuilder::new()
    .route(Route::url("/user/:user_id/home").get(user_handler))
    .build();
```

### Serving with hyper

Pass the router to `serve_connection` once per accepted connection. See
[`examples/example_server.rs`](examples/example_server.rs) for a complete
server:

```sh
cargo run --example example_server
curl localhost:8080/user/42/home
```

### Mounting inside axum

Use `axum::body::Body` for both body types, then nest the router under a
prefix or use it as the fallback:

```rust
use axum::body::Body;
use http::{Request, Response};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

fn user_handler(params: UrlParams, _: Request<Body>) -> Response<Body> {
    let id = params.captures.get(":user_id").cloned().unwrap_or_default();
    Response::new(Body::from(format!("user: {id}")))
}

let tree = RouterBuilder::new()
    .route(Route::url("/user/:user_id").get(user_handler))
    .build();

let app = axum::Router::new()
    .route("/health", axum::routing::get(|| async { "ok" }))
    .nest_service("/legacy", tree); // GET /legacy/user/7 -> "user: 7"
```

## Migrating from 0.1

- Requires hyper 1.x. `Server::bind(..).serve(router)` is gone; serve the
  router per connection (see the example).
- `Route`, `RouterBuilder` and `Router` are now generic over
  `<ReqBody, ResBody>`. Handlers take `Request<ReqBody>` and return
  `Response<ResBody>` rather than using `hyper::Body`.
- `Router` is now the request service itself. `Router.routes` is no longer
  public.
