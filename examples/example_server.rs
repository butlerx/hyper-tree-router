extern crate futures;
extern crate hyper;

use crate::futures::Future;
use hyper::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    server::Server,
    Body, Request, Response,
};
use hyper_tree_router::{Route, Router, RouterBuilder, UrlParams};

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

fn product_handler(url_params: UrlParams, _: Request<Body>) -> Response<Body> {
    let body = format!(
        "product {}",
        url_params
            .captures
            .get(":product_id")
            .unwrap_or(&"unknown user".to_string())
    );
    plain_text_response(body)
}

fn router_service() -> Result<Router, std::io::Error> {
    let router_builder = RouterBuilder::new()
        .add(Route::url("/user/:user_id/home").get(user_handler))
        .add(Route::url("/product/:product_id/info").get(product_handler))
        .build();

    Ok(router_builder)
}

fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let server = Server::bind(&addr)
        .serve(router_service)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server)
}
