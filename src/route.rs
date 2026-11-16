use super::parameters::UrlParams;
use http::{Method, Request, Response};
use std::{collections::HashMap, fmt, future::Future, pin::Pin, sync::Arc};

/// Boxed future returned by a [`Handler`].
pub type HandlerFuture<ResBody> = Pin<Box<dyn Future<Output = Response<ResBody>> + Send>>;

/// Type-erased async request handler.
///
/// `ReqBody` is the incoming request body type (for example `hyper::body::Incoming`
/// when serving with hyper directly, or `axum::body::Body` inside axum).
/// `ResBody` is the response body type the handler produces.
///
/// You don't build this directly: pass an `async fn` or a closure returning a future to
/// [`Route::get`] and friends.
pub type Handler<ReqBody, ResBody> =
    Arc<dyn Fn(UrlParams, Request<ReqBody>) -> HandlerFuture<ResBody> + Send + Sync>;

/// Define a Server Route.
/// Supports multiple method types per path. Handlers are async: pass an `async fn` or a
/// closure returning a future. Closures can capture shared state (e.g. an `Arc<AppState>`).
/// Optional URL Params can be defined with Path elements in the form `/:param_name` and then
/// later be looked up in the handler.
pub struct Route<ReqBody, ResBody> {
    /// Path to match.
    pub path: String,

    /// Request handlers.
    pub handlers: HashMap<Method, Handler<ReqBody, ResBody>>,
}

impl<ReqBody, ResBody> Route<ReqBody, ResBody> {
    /// Add handler for OPTIONS type requests.
    #[must_use]
    pub fn options<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        self.using(Method::OPTIONS, handler)
    }

    /// Add handler for GET type requests.
    #[must_use]
    pub fn get<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        self.using(Method::GET, handler)
    }

    /// Add handler for POST type requests.
    #[must_use]
    pub fn post<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        self.using(Method::POST, handler)
    }

    /// Add handler for PUT type requests.
    #[must_use]
    pub fn put<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        self.using(Method::PUT, handler)
    }

    /// Add handler for DELETE type requests.
    #[must_use]
    pub fn delete<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        self.using(Method::DELETE, handler)
    }

    /// Add handler for HEAD type requests.
    #[must_use]
    pub fn head<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        self.using(Method::HEAD, handler)
    }

    /// Add handler for TRACE type requests.
    #[must_use]
    pub fn trace<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        self.using(Method::TRACE, handler)
    }

    /// Add handler for CONNECT type requests.
    #[must_use]
    pub fn connect<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        self.using(Method::CONNECT, handler)
    }

    /// Add handler for PATCH type requests.
    #[must_use]
    pub fn patch<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        self.using(Method::PATCH, handler)
    }

    /// Create `Route` for a given url.
    /// URL Parameters can be defined in the form `:param_name` e.g.
    /// `/user/:user_id/info`
    #[must_use]
    pub fn url(path: &str) -> Self {
        Self {
            path: path.to_string(),
            handlers: HashMap::new(),
        }
    }

    fn using<F, Fut>(mut self, method: Method, handler: F) -> Self
    where
        F: Fn(UrlParams, Request<ReqBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResBody>> + Send + 'static,
    {
        let handler: Handler<ReqBody, ResBody> =
            Arc::new(move |params, request| Box::pin(handler(params, request)));
        self.handlers.insert(method, handler);
        self
    }
}

impl<ReqBody, ResBody> Default for Route<ReqBody, ResBody> {
    fn default() -> Self {
        Self::url("/")
    }
}

// Manual impl: `#[derive(Clone)]` would wrongly require `ReqBody: Clone + ResBody: Clone`.
impl<ReqBody, ResBody> Clone for Route<ReqBody, ResBody> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            handlers: self.handlers.clone(),
        }
    }
}

impl<ReqBody, ResBody> fmt::Debug for Route<ReqBody, ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Route {{methods: {:?}, path: {:?}}}",
            self.handlers.keys(),
            self.path
        )
    }
}
