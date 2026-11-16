mod builder;
mod parameters;
mod route;
mod router;

pub use self::{
    builder::RouterBuilder,
    parameters::UrlParams,
    route::{Handler, HandlerFuture, Route},
    router::{ErrorHandler, Router, ServiceFuture},
};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use http::{Method, Request, Response, StatusCode};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tower::ServiceExt;

    type Req = Request<()>;
    type Res = Response<String>;

    /// Handler that responds with a fixed label so tests can see which one ran.
    fn label(name: &'static str) -> impl Fn(UrlParams, Req) -> std::future::Ready<Res> + Clone {
        move |_, _| std::future::ready(Response::new(name.to_string()))
    }

    async fn echo_id(params: UrlParams, _: Req) -> Res {
        Response::new(params.captures.get(":id").cloned().unwrap_or_default())
    }

    fn test_router() -> Router<(), String> {
        RouterBuilder::new()
            .route(
                Route::url("/")
                    .get(label("get_root"))
                    .post(label("post_root")),
            )
            .route(
                Route::url("/hello")
                    .get(label("get_hello"))
                    .post(label("post_hello"))
                    .patch(label("patch_hello"))
                    .put(label("put_hello"))
                    .delete(label("delete_hello"))
                    .options(label("options_hello"))
                    .trace(label("trace_hello"))
                    .head(label("head_hello")),
            )
            .route(
                Route::url("/foo")
                    .get(label("get_foo"))
                    .post(label("post_foo")),
            )
            .route(Route::url("/bar").get(label("get_bar")))
            .route(Route::url("/foo/:id").get(echo_id))
            .route(Route::url("/bar/:id").get(label("param_bar")))
            .build()
    }

    async fn send(router: Router<(), String>, method: Method, uri: &str) -> (StatusCode, String) {
        let req = Request::builder().method(method).uri(uri).body(()).unwrap();
        let res = router.oneshot(req).await.unwrap();
        (res.status(), res.into_body())
    }

    #[tokio::test]
    async fn test_each_method_dispatches_to_its_handler() {
        let cases = [
            (Method::GET, "get_hello"),
            (Method::POST, "post_hello"),
            (Method::PUT, "put_hello"),
            (Method::PATCH, "patch_hello"),
            (Method::DELETE, "delete_hello"),
            (Method::OPTIONS, "options_hello"),
            (Method::TRACE, "trace_hello"),
            (Method::HEAD, "head_hello"),
        ];
        for (method, expected) in cases {
            let got = send(
                test_router(),
                method.clone(),
                "http://www.example.com/hello",
            )
            .await;
            assert_eq!(got, (StatusCode::OK, expected.to_string()), "{method}");
        }
    }

    #[tokio::test]
    async fn test_root_and_siblings() {
        assert_eq!(send(test_router(), Method::GET, "/").await.1, "get_root");
        assert_eq!(send(test_router(), Method::POST, "/").await.1, "post_root");
        assert_eq!(send(test_router(), Method::GET, "/foo").await.1, "get_foo");
        assert_eq!(send(test_router(), Method::GET, "/bar").await.1, "get_bar");
        assert_eq!(
            send(test_router(), Method::GET, "/bar/1").await.1,
            "param_bar"
        );
    }

    #[tokio::test]
    async fn test_url_path_parameters() {
        assert_eq!(
            send(test_router(), Method::GET, "/foo/bar").await,
            (StatusCode::OK, "bar".to_string())
        );
    }

    #[tokio::test]
    async fn test_no_route() {
        assert_eq!(
            send(test_router(), Method::GET, "/notfound").await,
            (StatusCode::NOT_FOUND, "Page Not Found".to_string())
        );
    }

    #[tokio::test]
    async fn test_method_not_allowed() {
        assert_eq!(
            send(test_router(), Method::DELETE, "/bar").await.0,
            StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    async fn test_method_not_allowed_lists_allowed_methods() {
        let req = Request::builder()
            .method(Method::DELETE)
            .uri("/foo")
            .body(())
            .unwrap();
        let res = test_router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(res.headers()[http::header::ALLOW], "GET, POST");
    }

    fn custom_errors() -> Router<(), String> {
        RouterBuilder::new()
            .route(Route::url("/only-get").get(label("ok")))
            .error_handler(|status| {
                let mut res = Response::new(format!("custom {}", status.as_u16()));
                *res.status_mut() = status;
                res
            })
            .build()
    }

    #[tokio::test]
    async fn test_custom_error_handler_not_found() {
        assert_eq!(
            send(custom_errors(), Method::GET, "/missing").await,
            (StatusCode::NOT_FOUND, "custom 404".to_string())
        );
    }

    #[tokio::test]
    async fn test_custom_error_handler_keeps_allow_header() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/only-get")
            .body(())
            .unwrap();
        let res = custom_errors().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(res.headers()[http::header::ALLOW], "GET");
        assert_eq!(res.into_body(), "custom 405");
    }

    #[tokio::test]
    async fn test_custom_error_handler_allow_header_is_preserved() {
        let router: Router<(), String> = RouterBuilder::new()
            .route(Route::url("/only-get").get(label("ok")))
            .error_handler(|status| {
                let mut res = Response::new(String::new());
                *res.status_mut() = status;
                res.headers_mut().insert(
                    http::header::ALLOW,
                    http::HeaderValue::from_static("CUSTOM"),
                );
                res
            })
            .build();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/only-get")
            .body(())
            .unwrap();
        let res = router.oneshot(req).await.unwrap();
        let allow: Vec<_> = res.headers().get_all(http::header::ALLOW).iter().collect();
        assert_eq!(allow, ["CUSTOM"]);
    }

    #[tokio::test]
    async fn test_closure_handler_captures_state() {
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&hits);
        let router: Router<(), String> = RouterBuilder::new()
            .route(Route::url("/count").get(move |_, _| {
                let counter = Arc::clone(&counter);
                async move {
                    let n = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    Response::new(n.to_string())
                }
            }))
            .build();

        assert_eq!(send(router.clone(), Method::GET, "/count").await.1, "1");
        assert_eq!(send(router, Method::GET, "/count").await.1, "2");
        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_router_is_send_sync_clone() {
        fn assert_bounds<T: Send + Sync + Clone + 'static>() {}
        assert_bounds::<Router<axum::body::Body, axum::body::Body>>();
    }
}
