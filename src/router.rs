use super::{parameters::UrlParams, route};
use futures::future::FutureResult;
use hyper::{header::CONTENT_LENGTH, service::Service, Body, Request, Response, StatusCode};
use prefix_tree_map::PrefixTreeMap;

type HttpResult<T> = Result<T, StatusCode>;

/// The default simple router service.
pub struct Router {
    routes: PrefixTreeMap<String, String, route::Route>,
    error_handler: fn(StatusCode) -> Response<Body>,
}

impl Router {
    pub fn new(routes: PrefixTreeMap<String, String, route::Route>) -> Self {
        Router {
            routes,
            error_handler: Self::default_error_handler,
        }
    }

    pub fn route(&self, request: &Request<Body>) -> HttpResult<(route::Handler, UrlParams)> {
        let path = request
            .uri()
            .path()
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let mut params = UrlParams::new();

        match self.routes.find_and_capture(&path, &mut params) {
            Some(route) => match route.handlers.get(request.method()) {
                Some(handler) => Ok((*handler, params)),
                _ => Err(StatusCode::METHOD_NOT_ALLOWED),
            },
            None => Err(StatusCode::NOT_FOUND),
        }
    }

    fn default_error_handler(status_code: StatusCode) -> Response<Body> {
        let (error_msg, error_code) = match status_code {
            StatusCode::NOT_FOUND => ("Page Not Found", StatusCode::NOT_FOUND),
            StatusCode::METHOD_NOT_ALLOWED => {
                ("method not supported", StatusCode::METHOD_NOT_ALLOWED)
            }
            StatusCode::NOT_IMPLEMENTED => ("not implemented", StatusCode::NOT_IMPLEMENTED),
            _ => ("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR),
        };
        Response::builder()
            .header(CONTENT_LENGTH, error_msg.len() as u64)
            .status(error_code)
            .body(Body::from(error_msg))
            .expect("Failed to construct a response")
    }
}

impl Service for Router {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Future = FutureResult<Response<Body>, hyper::Error>;

    fn call(&mut self, request: Request<Self::ReqBody>) -> Self::Future {
        futures::future::ok(match self.route(&request) {
            Ok((handler, url_params)) => handler(url_params, request),
            Err(status_code) => (self.error_handler)(status_code),
        })
    }
}
