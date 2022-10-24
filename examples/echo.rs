use futures_util::TryStreamExt;
use hyper::{Body, Request, Response, Server};
use hyper_tree_router::{Route, RouterBuilder, UrlParams};

/// Serve some instructions at /
async fn home(_: UrlParams, _: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from(
        "Try POSTing data to /echo such as: `curl localhost:8080/echo -XPOST -d 'hello world'`",
    )))
}

/// Simply echo the body back to the client.
async fn echo(_: UrlParams, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(req.into_body()))
}

/// Convert to uppercase before sending back to client using a stream.
async fn echo_uppercase(_: UrlParams, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let chunk_stream = req.into_body().map_ok(|chunk| {
        chunk
            .iter()
            .map(|byte| byte.to_ascii_uppercase())
            .collect::<Vec<u8>>()
    });
    Ok(Response::new(Body::wrap_stream(chunk_stream)))
}

// Reverse the entire body before sending back to the client.
//
// Since we don't know the end yet, we can't simply stream
// the chunks as they arrive as we did with the above uppercase endpoint.
// So here we do `.await` on the future, waiting on concatenating the full body,
// then afterwards the content can be reversed. Only then can we return a `Response`.
async fn echo_reversed(_: UrlParams, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let whole_body = hyper::body::to_bytes(req.into_body()).await?;

    let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
    Ok(Response::new(Body::from(reversed_body)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let router = RouterBuilder::new()
        .route(Route::url("/").get(home))
        .route(Route::url("/echo").post(echo))
        .route(Route::url("/echo/uppercase").post(echo_uppercase))
        .route(Route::url("/echo/reversed").post(echo_reversed))
        .build();
    let server = Server::bind(&addr).serve(router);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}
