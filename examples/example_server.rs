use http_body_util::Full;
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    server::conn::http1,
};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

type Body = Full<Bytes>;

fn plain_text_response(body: String) -> Response<Body> {
    Response::builder()
        .header(CONTENT_LENGTH, body.len() as u64)
        .header(CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .expect("Failed to construct response")
}

fn user_handler(url_params: UrlParams, _: Request<Incoming>) -> Response<Body> {
    let UrlParams { captures } = url_params;
    let body = format!(
        "user: {}",
        captures
            .get(":user_id")
            .map_or("unknown user", String::as_str)
    );
    plain_text_response(body)
}

fn product_handler(url_params: UrlParams, _: Request<Incoming>) -> Response<Body> {
    let UrlParams { captures } = url_params;
    let body = format!(
        "product {}",
        captures
            .get(":product_id")
            .map_or("unknown product", String::as_str)
    );
    plain_text_response(body)
}

fn hello_handler(_: UrlParams, _: Request<Incoming>) -> Response<Body> {
    plain_text_response("Hello World".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "0.0.0.0:8080";
    let router = RouterBuilder::new()
        .route(Route::url("/").get(hello_handler))
        .route(Route::url("/user/:user_id/home").get(user_handler))
        .route(Route::url("/product/:product_id/info").get(product_handler))
        .build();

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{addr}");
    loop {
        let (stream, _) = listener.accept().await?;
        let router = router.clone();
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), router)
                .await
            {
                eprintln!("connection error: {err}");
            }
        });
    }
}
