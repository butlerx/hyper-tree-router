mod builder;
mod extract;
mod parameters;
mod route;
mod router;

#[cfg(feature = "serde")]
pub use self::extract::Params;
pub use self::{
    builder::RouterBuilder,
    extract::{FromUrlParams, ParamsRejection},
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
        Response::new(params.get("id").unwrap_or_default().to_string())
    }

    fn test_router() -> Router<(), String> {
        RouterBuilder::default()
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
        assert_eq!(res.headers()[http::header::ALLOW], "GET, HEAD, POST");
    }

    fn custom_errors() -> Router<(), String> {
        RouterBuilder::default()
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
        assert_eq!(res.headers()[http::header::ALLOW], "GET, HEAD");
        assert_eq!(res.into_body(), "custom 405");
    }

    #[tokio::test]
    async fn test_custom_error_handler_allow_header_is_preserved() {
        let router: Router<(), String> = RouterBuilder::default()
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
        let router: Router<(), String> = RouterBuilder::default()
            .route(Route::url("/count").get(move |_: UrlParams, _| {
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

    fn files_router() -> Router<(), String> {
        RouterBuilder::default()
            .route(
                Route::url("/files/*path").get(|p: UrlParams, _: Req| async move {
                    Response::new(format!("catch-all {}", p.get("path").unwrap_or_default()))
                }),
            )
            .route(Route::url("/files/readme").get(label("exact readme")))
            .route(
                Route::url("/files/:name/raw").get(|p: UrlParams, _: Req| async move {
                    Response::new(format!("raw {}", p.get("name").unwrap_or_default()))
                }),
            )
            .route(
                Route::url("/files/deep/*rest").get(|p: UrlParams, _: Req| async move {
                    Response::new(format!("deep {}", p.get("rest").unwrap_or_default()))
                }),
            )
            .build()
    }

    #[tokio::test]
    async fn test_catch_all_captures_rest_of_path() {
        assert_eq!(
            send(files_router(), Method::GET, "/files/a/b/c.txt").await,
            (StatusCode::OK, "catch-all a/b/c.txt".to_string())
        );
    }

    #[tokio::test]
    async fn test_exact_and_param_routes_beat_catch_all() {
        assert_eq!(
            send(files_router(), Method::GET, "/files/readme").await.1,
            "exact readme"
        );
        assert_eq!(
            send(files_router(), Method::GET, "/files/x/raw").await.1,
            "raw x"
        );
    }

    #[tokio::test]
    async fn test_longest_catch_all_prefix_wins() {
        assert_eq!(
            send(files_router(), Method::GET, "/files/deep/a/b").await.1,
            "deep a/b"
        );
    }

    #[tokio::test]
    async fn test_catch_all_needs_at_least_one_segment() {
        assert_eq!(
            send(files_router(), Method::GET, "/files").await.0,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_catch_all_rejects_trailing_slash_and_root() {
        assert_eq!(
            send(files_router(), Method::GET, "/files/").await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            send(files_router(), Method::GET, "/files//").await.0,
            StatusCode::NOT_FOUND
        );

        let root: Router<(), String> = RouterBuilder::default()
            .route(Route::url("/*rest").get(|p: UrlParams, _: Req| async move {
                Response::new(p.get("rest").unwrap_or_default().to_string())
            }))
            .build();
        assert_eq!(
            send(root.clone(), Method::GET, "/").await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            send(root, Method::GET, "/a/b").await,
            (StatusCode::OK, "a/b".to_string())
        );
    }

    #[tokio::test]
    async fn test_deep_catch_all_falls_back_to_shallower_one() {
        // `/files/deep/` has an empty remainder for `deep/*rest`, so `/files/*path` takes it.
        assert_eq!(
            send(files_router(), Method::GET, "/files/deep/").await.1,
            "catch-all deep/"
        );
    }

    #[test]
    #[should_panic(expected = "catch-all `*` must be the last segment")]
    fn test_catch_all_must_be_last() {
        let _: RouterBuilder<(), String> =
            RouterBuilder::default().route(Route::url("/a/*rest/b").get(label("x")));
    }

    #[tokio::test]
    async fn test_head_falls_back_to_get() {
        assert_eq!(
            send(test_router(), Method::HEAD, "/foo").await,
            (StatusCode::OK, "get_foo".to_string())
        );
    }

    #[tokio::test]
    async fn test_explicit_head_handler_wins_over_get() {
        assert_eq!(
            send(test_router(), Method::HEAD, "/hello").await.1,
            "head_hello"
        );
    }

    #[test]
    fn test_params_use_bare_names() {
        let req = Request::builder().uri("/foo/42").body(()).unwrap();
        let (_, params) = test_router().route(&req).unwrap();
        assert_eq!(params.get("id"), Some("42"));
        assert_eq!(params.get(":id"), None);
    }

    #[test]
    fn test_params_keep_path_order_and_duplicates() {
        let router: Router<(), String> = RouterBuilder::default()
            .route(Route::url("/:org/:repo/x/:org").get(label("x")))
            .build();
        let req = Request::builder()
            .uri("/acme/widget/x/other")
            .body(())
            .unwrap();
        let (_, params) = router.route(&req).unwrap();
        assert_eq!(
            params.iter().collect::<Vec<_>>(),
            [("org", "acme"), ("repo", "widget"), ("org", "other")]
        );
        assert_eq!(params.get("org"), Some("acme"));
        assert_eq!(params.len(), 3);
        assert!(UrlParams::default().is_empty());
    }

    #[test]
    fn test_router_is_send_sync_clone() {
        fn assert_bounds<T: Send + Sync + Clone + 'static>() {}
        assert_bounds::<Router<axum::body::Body, axum::body::Body>>();
    }
}

#[cfg(all(test, feature = "serde"))]
mod typed_params_tests {
    use super::*;
    use http::{Request, Response, StatusCode};
    use serde::Deserialize;
    use tower::ServiceExt;

    type Req = Request<()>;
    type Res = Response<String>;

    #[derive(Deserialize)]
    struct UserPath {
        user_id: u64,
        tab: Option<String>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Kind {
        Issues,
        Pulls,
    }

    async fn user(Params(p): Params<UserPath>, _: Req) -> Res {
        Response::new(format!("user {} tab {:?}", p.user_id, p.tab))
    }

    async fn repo(Params((org, repo)): Params<(String, String)>, _: Req) -> Res {
        Response::new(format!("{org}/{repo}"))
    }

    async fn kind(Params((org, kind)): Params<(String, Kind)>, _: Req) -> Res {
        Response::new(format!("{org} {kind:?}"))
    }

    async fn single(Params(id): Params<u32>, _: Req) -> Res {
        Response::new(format!("id {id}"))
    }

    async fn raw(params: UrlParams, _: Req) -> Res {
        Response::new(format!("raw {}", params.len()))
    }

    fn router() -> Router<(), String> {
        RouterBuilder::default()
            .route(Route::url("/user/:user_id/:tab").get(user))
            .route(Route::url("/repo/:org/:repo").get(repo))
            .route(Route::url("/kind/:org/:kind").get(kind))
            .route(Route::url("/single/:id").get(single))
            .route(Route::url("/raw/:a/:b").get(raw))
            .route(
                Route::url("/files/*path")
                    .get(|Params(path): Params<String>, _: Req| async move { Response::new(path) }),
            )
            .build()
    }

    async fn get(uri: &str) -> (StatusCode, String) {
        let res = router()
            .oneshot(Request::builder().uri(uri).body(()).unwrap())
            .await
            .unwrap();
        (res.status(), res.into_body())
    }

    #[tokio::test]
    async fn struct_reads_params_by_name() {
        assert_eq!(
            get("/user/42/settings").await,
            (
                StatusCode::OK,
                r#"user 42 tab Some("settings")"#.to_string()
            )
        );
    }

    #[tokio::test]
    async fn tuple_reads_params_in_path_order() {
        assert_eq!(get("/repo/acme/widget").await.1, "acme/widget");
    }

    #[tokio::test]
    async fn unit_enum_variant() {
        assert_eq!(get("/kind/acme/pulls").await.1, "acme Pulls");
        assert_eq!(get("/kind/acme/nope").await.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn single_value_and_catch_all() {
        assert_eq!(get("/single/7").await.1, "id 7");
        assert_eq!(get("/files/a/b.txt").await.1, "a/b.txt");
    }

    #[tokio::test]
    async fn untyped_params_still_work() {
        assert_eq!(get("/raw/x/y").await.1, "raw 2");
    }

    #[tokio::test]
    async fn parse_failure_is_bad_request_without_calling_handler() {
        assert_eq!(
            get("/user/abc/settings").await,
            (
                StatusCode::BAD_REQUEST,
                "Invalid URL parameters".to_string()
            )
        );
        assert_eq!(get("/single/-1").await.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn bad_request_goes_through_custom_error_handler() {
        let router: Router<(), String> = RouterBuilder::default()
            .route(Route::url("/single/:id").get(single))
            .error_handler(|status| {
                let mut res = Response::new(format!("custom {}", status.as_u16()));
                *res.status_mut() = status;
                res
            })
            .build();
        let res = router
            .oneshot(Request::builder().uri("/single/x").body(()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.into_body(), "custom 400");
    }

    #[test]
    fn rejection_explains_the_failure() {
        let mut params = UrlParams::default();
        params.push("id".into(), "x".into());
        let err = <Params<u32>>::from_url_params(params).unwrap_err();
        assert!(err.to_string().contains("invalid URL parameters"), "{err}");
        assert!(err.to_string().contains("\"x\""), "{err}");
    }
}
