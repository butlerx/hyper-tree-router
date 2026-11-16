mod builder;
mod parameters;
mod route;
mod router;

pub use self::{builder::RouterBuilder, parameters::UrlParams, route::Route, router::Router};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use http::{Method, Request, Response, StatusCode, Uri};
    use std::str::FromStr;
    use tower::ServiceExt;

    type Req = Request<()>;
    type Res = Response<String>;

    fn handle_get_hello(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_post_hello(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_delete_hello(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_options_hello(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_put_hello(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_head_hello(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_trace_hello(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_patch_hello(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_get_root(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_post_root(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_get_foo(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_post_foo(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_get_bar(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_param_foo(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }
    fn handle_param_bar(_: UrlParams, _: Req) -> Res {
        unimplemented!()
    }

    fn test_router() -> Router<(), String> {
        RouterBuilder::new()
            .route(Route::url("/").get(handle_get_root).post(handle_post_root))
            .route(
                Route::url("/hello")
                    .get(handle_get_hello)
                    .post(handle_post_hello)
                    .patch(handle_patch_hello)
                    .put(handle_put_hello)
                    .delete(handle_delete_hello)
                    .options(handle_options_hello)
                    .trace(handle_trace_hello)
                    .head(handle_head_hello),
            )
            .route(Route::url("/foo").get(handle_get_foo).post(handle_post_foo))
            .route(Route::url("/bar").get(handle_get_bar))
            .route(Route::url("/foo/:id").get(handle_param_foo))
            .route(Route::url("/bar/:id").get(handle_param_bar))
            .build()
    }

    #[test]
    fn test_get_route() {
        let request = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(())
            .unwrap();
        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert_eq!(handler as usize, handle_get_hello as fn(_, _) -> _ as usize);
    }

    #[test]
    fn test_post_route() {
        let request = Request::builder()
            .method(Method::POST)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert_eq!(
            handler as usize,
            handle_post_hello as fn(_, _) -> _ as usize
        );
    }

    #[test]
    fn test_delete_route() {
        let request = Request::builder()
            .method(Method::DELETE)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert_eq!(
            handler as usize,
            handle_delete_hello as fn(_, _) -> _ as usize
        );
    }

    #[test]
    fn test_options_route() {
        let request = Request::builder()
            .method(Method::OPTIONS)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert_eq!(
            handler as usize,
            handle_options_hello as fn(_, _) -> _ as usize
        );
    }

    #[test]
    fn test_put_route() {
        let request = Request::builder()
            .method(Method::PUT)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert_eq!(handler as usize, handle_put_hello as fn(_, _) -> _ as usize);
    }

    #[test]
    fn test_head_route() {
        let request = Request::builder()
            .method(Method::HEAD)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert_eq!(
            handler as usize,
            handle_head_hello as fn(_, _) -> _ as usize
        );
    }

    #[test]
    fn test_trace_route() {
        let request = Request::builder()
            .method(Method::TRACE)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert_eq!(
            handler as usize,
            handle_trace_hello as fn(_, _) -> _ as usize
        );
    }

    #[test]
    fn test_patch_route() {
        let request = Request::builder()
            .method(Method::PATCH)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert_eq!(
            handler as usize,
            handle_patch_hello as fn(_, _) -> _ as usize
        );
    }

    #[test]
    fn test_no_route() {
        let request = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_str("http://www.example.com/notfound").unwrap())
            .body(())
            .unwrap();

        let router = test_router();
        match router.route(&request) {
            Ok(_) => panic!("Expected an error, but got a handler instead"),
            Err(e) => assert_eq!(e, StatusCode::NOT_FOUND),
        }
    }

    #[test]
    fn test_url_path_parameters() {
        let request = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_str("http://www.example.com/foo/bar").unwrap())
            .body(())
            .unwrap();

        let router = test_router();
        let (handler, params) = router.route(&request).unwrap();
        assert_eq!(handler as usize, handle_param_foo as fn(_, _) -> _ as usize);
        assert!(params.captures.contains_key(":id"));
        assert_eq!(params.captures.get(":id"), Some(&"bar".to_string()));
    }

    #[expect(clippy::needless_pass_by_value, reason = "handler signature is fixed")]
    fn handle_ok(params: UrlParams, _: Req) -> Res {
        Response::new(params.captures.get(":id").cloned().unwrap_or_default())
    }

    fn service_router() -> Router<(), String> {
        RouterBuilder::new()
            .route(Route::url("/item/:id").get(handle_ok))
            .build()
    }

    fn get(uri: &str) -> Req {
        Request::builder().uri(uri).body(()).unwrap()
    }

    #[tokio::test]
    async fn test_service_calls_handler() {
        let res = service_router().oneshot(get("/item/42")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.body(), "42");
    }

    #[tokio::test]
    async fn test_service_not_found() {
        let res = service_router().oneshot(get("/nope")).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert_eq!(res.body(), "Page Not Found");
    }

    #[tokio::test]
    async fn test_service_method_not_allowed() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/item/1")
            .body(())
            .unwrap();
        let res = service_router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_router_is_send_sync_clone() {
        fn assert_bounds<T: Send + Sync + Clone + 'static>() {}
        assert_bounds::<Router<axum::body::Body, axum::body::Body>>();
    }
}
