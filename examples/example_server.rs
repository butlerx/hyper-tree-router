use hyper::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Body, Request, Response, Server,
};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

fn plain_text_response(body: String) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::builder()
        .header(CONTENT_LENGTH, body.len() as u64)
        .header(CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .expect("Failed to construct response"))
}

async fn user_handler(
    url_params: UrlParams,
    _: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let body = format!(
        "user: {}",
        url_params
            .captures
            .get(":user_id")
            .unwrap_or(&"unknown user".to_string())
    );
    plain_text_response(body)
}

async fn product_handler(
    url_params: UrlParams,
    _: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let body = format!(
        "product {}",
        url_params
            .captures
            .get(":product_id")
            .unwrap_or(&"unknown user".to_string())
    );
    plain_text_response(body)
}

async fn hello_handler(_: UrlParams, _: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    plain_text_response("Hello World".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let router = RouterBuilder::new()
        .route(Route::url("/").get(hello_handler))
        .route(Route::url("/user/:user_id/home").get(user_handler))
        .route(Route::url("/product/:product_id/info").get(product_handler))
        .build();
    let server = Server::bind(&addr).serve(router);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}
