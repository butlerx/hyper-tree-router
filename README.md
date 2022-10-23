# Hyper Tree Router

Routing middleware for Hyper http library using Prefix tree (trie) for path
finding. Gives the ability to define routes and handlers for given request path

## Usage

To use the library add to your `Cargo.toml`

```toml
hyper = { version = "^0.14", features = ["http1", "server", "tcp"] }
hyper-tree-router = "^0.1"
```

Then you can define handlers for a given route as follows. The below code
creates two `Route` and creates a hyper server for them.

`/` with `hello_handler` that will return "Hello world" when you open the server

`/user/:user_id/home` with the `user_handler` the `url_params` contains a
hashmap of url parameters you can look up and respond to.

```rust
use hyper::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Body, Request, Response, Server,
};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

fn plain_text_response(body: String) -> Response<Body> {
    Response::builder()
        .header(CONTENT_LENGTH, body.len() as u64)
        .header(CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .expect("Failed to construct response")
}

fn user_handler(url_params: UrlParams, _: Request<Body>) -> Response<Body> {
    let body = format!(
        "user: {}",
        url_params
            .captures
            .get(":user_id")
            .unwrap_or(&"unknown user".to_string())
    );
    plain_text_response(body)
}

fn hello_handler(_: UrlParams, _: Request<Body>) -> Response<Body> {
    plain_text_response("Hello World".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let router = RouterBuilder::new()
        .route(Route::url("/user/:user_id/home").get(user_handler))
        .route(Route::url("/").get(hello_hanlder))
        .build();
    let server = Server::bind(&addr).serve(router);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}
```
