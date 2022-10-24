use super::{parameters::UrlParams, route};
use hyper::{header::CONTENT_LENGTH, service::Service, Body, Request, Response, StatusCode};
use prefix_tree_map::PrefixTreeMap;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

type HttpResult<T> = Result<T, StatusCode>;

/// Router service to be passed to hyper `Server.bind().serve()`.
/// This should be build with RouterBuilder.
pub struct Router {
    pub routes: PrefixTreeMap<String, String, route::Route>,
}

impl Router {
    /// Create a new Router with a given prefix tree.
    pub fn new(routes: PrefixTreeMap<String, String, route::Route>) -> Self {
        Self { routes }
    }
}

pub struct RouterSvc {
    routes: PrefixTreeMap<String, String, route::Route>,
}

impl RouterSvc {
    pub fn new(routes: PrefixTreeMap<String, String, route::Route>) -> Self {
        Self { routes }
    }

    pub fn route(&self, request: &Request<Body>) -> HttpResult<(&route::Handler, UrlParams)> {
        let path = request
            .uri()
            .path()
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let mut params = UrlParams::new();

        match self.routes.find_and_capture(&path, &mut params) {
            Some(route) => match route.handlers.get(request.method()) {
                Some(handler) => Ok((handler, params)),
                _ => Err(StatusCode::METHOD_NOT_ALLOWED),
            },
            None => Err(StatusCode::NOT_FOUND),
        }
    }

    fn default_error_handler(status_code: StatusCode) -> Result<Response<Body>, hyper::Error> {
        let (error_msg, error_code) = match status_code {
            StatusCode::NOT_FOUND => ("Page Not Found", StatusCode::NOT_FOUND),
            StatusCode::METHOD_NOT_ALLOWED => {
                ("method not supported", StatusCode::METHOD_NOT_ALLOWED)
            }
            StatusCode::NOT_IMPLEMENTED => ("not implemented", StatusCode::NOT_IMPLEMENTED),
            _ => ("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR),
        };
        Ok(Response::builder()
            .header(CONTENT_LENGTH, error_msg.len() as u64)
            .status(error_code)
            .body(Body::from(error_msg))
            .expect("Failed to construct a response"))
    }
}

impl Service<Request<Body>> for RouterSvc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        match self.route(&request) {
            Ok((&handler, url_params)) => Box::pin(handler(url_params, request)),
            Err(status_code) => Box::pin(async move { Self::default_error_handler(status_code) }),
        }
    }
}

impl<T> Service<T> for Router {
    type Response = RouterSvc;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _: T) -> Self::Future {
        let routes = self.routes.clone();
        Box::pin(async move { Ok(RouterSvc::new(routes)) })
    }
}
