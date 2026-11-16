//! Mounting the tree router inside an axum application.

use axum::body::Body;
use http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use hyper_tree_router::{Route, RouterBuilder, UrlParams};
use tower::ServiceExt;

async fn user_handler(params: UrlParams, _: Request<Body>) -> Response<Body> {
    let id = params.get("user_id").unwrap_or_default();
    Response::new(Body::from(format!("user: {id}")))
}

fn app() -> axum::Router {
    let tree = RouterBuilder::default()
        .route(Route::url("/user/:user_id").get(user_handler))
        .build();

    axum::Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .nest_service("/legacy", tree)
}

async fn call(uri: &str) -> (StatusCode, String) {
    let res = app()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn nested_route_reaches_tree_router() {
    assert_eq!(
        call("/legacy/user/7").await,
        (StatusCode::OK, "user: 7".into())
    );
}

#[tokio::test]
async fn nested_miss_uses_tree_router_404() {
    assert_eq!(
        call("/legacy/missing").await,
        (StatusCode::NOT_FOUND, "Page Not Found".into())
    );
}

#[tokio::test]
async fn axum_routes_still_work() {
    assert_eq!(call("/health").await, (StatusCode::OK, "ok".into()));
}
