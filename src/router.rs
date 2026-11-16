use super::{
    parameters::UrlParams,
    route::{Handler, HandlerFuture, Route},
};
use http::{
    HeaderValue, Request, Response, StatusCode,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
};
use prefix_tree_map::PrefixTreeMap;
use std::{
    convert::Infallible,
    fmt,
    future::{Future, ready},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// Future returned by the `Service` impls.
///
/// Wraps the handler's boxed future and maps its output to `Ok`, so each request
/// costs exactly one allocation (the handler future itself).
pub struct ServiceFuture<ResBody> {
    inner: HandlerFuture<ResBody>,
}

impl<ResBody> Future for ServiceFuture<ResBody> {
    type Output = Result<Response<ResBody>, Infallible>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx).map(Ok)
    }
}

impl<ResBody> fmt::Debug for ServiceFuture<ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ServiceFuture").finish_non_exhaustive()
    }
}

type Routes<ReqBody, ResBody> = PrefixTreeMap<String, String, Route<ReqBody, ResBody>>;

/// Router service. Build it with `RouterBuilder`.
///
/// It implements both `tower::Service` and `hyper::service::Service`, so it can be
/// served directly by hyper, wrapped in tower middleware, or mounted inside axum with
/// `axum::Router::nest_service` / `fallback_service`.
///
/// Cloning is cheap: the route tree is shared behind an `Arc`.
pub struct Router<ReqBody, ResBody> {
    routes: Arc<Routes<ReqBody, ResBody>>,
}

impl<ReqBody, ResBody> Router<ReqBody, ResBody> {
    /// Create a new Router with a given prefix tree.
    #[must_use]
    pub fn new(routes: Routes<ReqBody, ResBody>) -> Self {
        Self {
            routes: Arc::new(routes),
        }
    }

    pub(crate) fn route(
        &self,
        request: &Request<ReqBody>,
    ) -> Result<(Handler<ReqBody, ResBody>, UrlParams), StatusCode> {
        let path = request
            .uri()
            .path()
            .split('/')
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut params = UrlParams::new();

        match self.routes.find_and_capture(&path, &mut params) {
            Some(route) => match route.handlers.get(request.method()) {
                Some(handler) => Ok((Arc::clone(handler), params)),
                None => Err(StatusCode::METHOD_NOT_ALLOWED),
            },
            None => Err(StatusCode::NOT_FOUND),
        }
    }
}

impl<ReqBody, ResBody> Router<ReqBody, ResBody>
where
    ResBody: From<&'static str> + Send + 'static,
{
    fn handle(&self, request: Request<ReqBody>) -> HandlerFuture<ResBody> {
        match self.route(&request) {
            Ok((handler, url_params)) => handler(url_params, request),
            Err(status_code) => Box::pin(ready(Self::error_response(status_code))),
        }
    }

    fn error_response(status_code: StatusCode) -> Response<ResBody> {
        let (error_msg, error_code) = match status_code {
            StatusCode::NOT_FOUND => ("Page Not Found", StatusCode::NOT_FOUND),
            StatusCode::METHOD_NOT_ALLOWED => {
                ("method not supported", StatusCode::METHOD_NOT_ALLOWED)
            }
            StatusCode::NOT_IMPLEMENTED => ("not implemented", StatusCode::NOT_IMPLEMENTED),
            _ => ("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR),
        };
        let mut response = Response::new(ResBody::from(error_msg));
        *response.status_mut() = error_code;
        let headers = response.headers_mut();
        headers.insert(CONTENT_LENGTH, HeaderValue::from(error_msg.len()));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        response
    }
}

// Manual impl: `#[derive(Clone)]` would wrongly require `ReqBody: Clone + ResBody: Clone`.
impl<ReqBody, ResBody> Clone for Router<ReqBody, ResBody> {
    fn clone(&self) -> Self {
        Self {
            routes: Arc::clone(&self.routes),
        }
    }
}

impl<ReqBody, ResBody> fmt::Debug for Router<ReqBody, ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Router").finish_non_exhaustive()
    }
}

impl<ReqBody, ResBody> tower_service::Service<Request<ReqBody>> for Router<ReqBody, ResBody>
where
    ResBody: From<&'static str> + Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = Infallible;
    type Future = ServiceFuture<ResBody>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        ServiceFuture {
            inner: self.handle(request),
        }
    }
}

impl<ReqBody, ResBody> hyper::service::Service<Request<ReqBody>> for Router<ReqBody, ResBody>
where
    ResBody: From<&'static str> + Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = Infallible;
    type Future = ServiceFuture<ResBody>;

    fn call(&self, request: Request<ReqBody>) -> Self::Future {
        ServiceFuture {
            inner: self.handle(request),
        }
    }
}
