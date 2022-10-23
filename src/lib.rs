mod builder;
mod parameters;
mod route;
mod router;

pub use self::{builder::RouterBuilder, parameters::UrlParams, route::Route, router::Router};

#[cfg(test)]
mod integration_tests {
    use super::router::RouterSvc;
    use super::*;
    use hyper::{Body, Method, Request, Response, Uri};
    use std::str::FromStr;

    fn handle_get_hello(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_post_hello(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_delete_hello(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_options_hello(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_put_hello(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_head_hello(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_trace_hello(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_patch_hello(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_get_root(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_post_root(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_get_foo(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_post_foo(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_get_bar(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_param_foo(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }
    fn handle_param_bar(_: UrlParams, _: Request<Body>) -> Response<Body> {
        unimplemented!()
    }

    fn test_router() -> RouterSvc {
        let router = RouterBuilder::new()
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
            .build();

        let routes = router.routes;
        RouterSvc::new(routes)
    }

    #[test]
    fn test_get_route() {
        let request = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(Body::empty())
            .unwrap();
        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert!(handler as fn(_, _) -> _ == handle_get_hello as fn(_, _) -> _);
    }

    #[test]
    fn test_post_route() {
        let request = Request::builder()
            .method(Method::POST)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(Body::empty())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert!(handler as fn(_, _) -> _ == handle_post_hello as fn(_, _) -> _);
    }

    #[test]
    fn test_delete_route() {
        let request = Request::builder()
            .method(Method::DELETE)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(Body::empty())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert!(handler as fn(_, _) -> _ == handle_delete_hello as fn(_, _) -> _);
    }

    #[test]
    fn test_options_route() {
        let request = Request::builder()
            .method(Method::OPTIONS)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(Body::empty())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert!(handler as fn(_, _) -> _ == handle_options_hello as fn(_, _) -> _);
    }

    #[test]
    fn test_put_route() {
        let request = Request::builder()
            .method(Method::PUT)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(Body::empty())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert!(handler as fn(_, _) -> _ == handle_put_hello as fn(_, _) -> _);
    }

    #[test]
    fn test_head_route() {
        let request = Request::builder()
            .method(Method::HEAD)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(Body::empty())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert!(handler as fn(_, _) -> _ == handle_head_hello as fn(_, _) -> _);
    }

    #[test]
    fn test_trace_route() {
        let request = Request::builder()
            .method(Method::TRACE)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(Body::empty())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert!(handler as fn(_, _) -> _ == handle_trace_hello as fn(_, _) -> _);
    }

    #[test]
    fn test_patch_route() {
        let request = Request::builder()
            .method(Method::PATCH)
            .uri(Uri::from_str("http://www.example.com/hello").unwrap())
            .body(Body::empty())
            .unwrap();

        let router = test_router();
        let (handler, _) = router.route(&request).unwrap();
        assert!(handler as fn(_, _) -> _ == handle_patch_hello as fn(_, _) -> _);
    }

    #[test]
    fn test_no_route() {
        let request = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_str("http://www.example.com/notfound").unwrap())
            .body(Body::empty())
            .unwrap();

        let router = test_router();
        match router.route(&request) {
            Ok(_) => panic!("Expected an error, but got a handler instead"),
            Err(e) => assert_eq!(e, hyper::StatusCode::NOT_FOUND),
        }
    }

    #[test]
    fn test_url_path_parameters() {
        let request = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_str("http://www.example.com/foo/bar").unwrap())
            .body(Body::empty())
            .unwrap();

        let router = test_router();
        let (handler, params) = router.route(&request).unwrap();
        assert!(handler as fn(_, _) -> _ == handle_param_foo as fn(_, _) -> _);
        assert!(params.captures.contains_key(":id"));
        assert!(params.captures.get(":id") == Some(&"bar".to_string()));
    }
}
