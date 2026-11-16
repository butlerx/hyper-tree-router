use super::{
    parameters::UrlParams,
    route::{Handler, HandlerFuture, Route},
};
use http::{
    HeaderValue, Method, Request, Response, StatusCode,
    header::{ALLOW, CONTENT_LENGTH, CONTENT_TYPE},
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

/// Builds the response for a routing failure (`404 Not Found` or
/// `405 Method Not Allowed`). Set one with `RouterBuilder::error_handler`.
pub type ErrorHandler<ResBody> = Arc<dyn Fn(StatusCode) -> Response<ResBody> + Send + Sync>;

/// Why a request didn't reach a handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RouteError {
    NotFound,
    /// The path matched, but not the method. Holds the methods the path does accept.
    MethodNotAllowed(Vec<Method>),
}

/// Router service. Build it with `RouterBuilder`.
///
/// It implements both `tower::Service` and `hyper::service::Service`, so it can be
/// served directly by hyper, wrapped in tower middleware, or mounted inside axum with
/// `axum::Router::nest_service` / `fallback_service`.
///
/// Cloning is cheap: the route tree is shared behind an `Arc`.
pub struct Router<ReqBody, ResBody> {
    routes: Arc<Routes<ReqBody, ResBody>>,
    error_handler: Option<ErrorHandler<ResBody>>,
}

impl<ReqBody, ResBody> Router<ReqBody, ResBody> {
    /// Create a new Router with a given prefix tree.
    #[must_use]
    pub fn new(routes: Routes<ReqBody, ResBody>) -> Self {
        Self {
            routes: Arc::new(routes),
            error_handler: None,
        }
    }

    pub(crate) fn with_error_handler(mut self, error_handler: ErrorHandler<ResBody>) -> Self {
        self.error_handler = Some(error_handler);
        self
    }

    pub(crate) fn route(
        &self,
        request: &Request<ReqBody>,
    ) -> Result<(Handler<ReqBody, ResBody>, UrlParams), RouteError> {
        let path = request
            .uri()
            .path()
            .split('/')
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut params = UrlParams::new();

        let route = self
            .routes
            .find_and_capture(&path, &mut params)
            .ok_or(RouteError::NotFound)?;
        if let Some(handler) = route.handlers.get(request.method()) {
            return Ok((Arc::clone(handler), params));
        }
        let mut allowed: Vec<Method> = route.handlers.keys().cloned().collect();
        allowed.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        Err(RouteError::MethodNotAllowed(allowed))
    }
}

impl<ReqBody, ResBody> Router<ReqBody, ResBody>
where
    ResBody: From<&'static str> + Send + 'static,
{
    fn handle(&self, request: Request<ReqBody>) -> HandlerFuture<ResBody> {
        match self.route(&request) {
            Ok((handler, url_params)) => handler(url_params, request),
            Err(error) => Box::pin(ready(self.error_response(&error))),
        }
    }

    fn error_response(&self, error: &RouteError) -> Response<ResBody> {
        let status = match error {
            RouteError::NotFound => StatusCode::NOT_FOUND,
            RouteError::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
        };
        let mut response = match &self.error_handler {
            Some(handler) => handler(status),
            None => Self::default_error_response(status),
        };
        // RFC 9110 §15.5.6: a 405 MUST list the supported methods. Added after the
        // custom handler runs so users don't have to remember it, unless they set it.
        if let RouteError::MethodNotAllowed(allowed) = error {
            let value = allowed
                .iter()
                .map(Method::as_str)
                .collect::<Vec<_>>()
                .join(", ");
            if let Ok(value) = HeaderValue::from_str(&value) {
                response.headers_mut().entry(ALLOW).or_insert(value);
            }
        }
        response
    }

    fn default_error_response(status: StatusCode) -> Response<ResBody> {
        let error_msg = match status {
            StatusCode::NOT_FOUND => "Page Not Found",
            StatusCode::METHOD_NOT_ALLOWED => "method not supported",
            _ => "Internal Server Error",
        };
        let mut response = Response::new(ResBody::from(error_msg));
        *response.status_mut() = status;
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
            error_handler: self.error_handler.clone(),
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
