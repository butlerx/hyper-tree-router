use super::parameters::UrlParams;
use http::{Method, Request, Response};
use std::{collections::HashMap, fmt};

/// Request handler.
///
/// `ReqBody` is the incoming request body type (for example `hyper::body::Incoming`
/// when serving with hyper directly, or `axum::body::Body` inside axum).
/// `ResBody` is the response body type the handler produces.
pub type Handler<ReqBody, ResBody> = fn(UrlParams, Request<ReqBody>) -> Response<ResBody>;

/// Define a Server Route.
/// Supports multiple method types per path.
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
    pub fn options(self, handler: Handler<ReqBody, ResBody>) -> Self {
        self.using(Method::OPTIONS, handler)
    }

    /// Add handler for GET type requests.
    #[must_use]
    pub fn get(self, handler: Handler<ReqBody, ResBody>) -> Self {
        self.using(Method::GET, handler)
    }

    /// Add handler for POST type requests.
    #[must_use]
    pub fn post(self, handler: Handler<ReqBody, ResBody>) -> Self {
        self.using(Method::POST, handler)
    }

    /// Add handler for PUT type requests.
    #[must_use]
    pub fn put(self, handler: Handler<ReqBody, ResBody>) -> Self {
        self.using(Method::PUT, handler)
    }

    /// Add handler for DELETE type requests.
    #[must_use]
    pub fn delete(self, handler: Handler<ReqBody, ResBody>) -> Self {
        self.using(Method::DELETE, handler)
    }

    /// Add handler for HEAD type requests.
    #[must_use]
    pub fn head(self, handler: Handler<ReqBody, ResBody>) -> Self {
        self.using(Method::HEAD, handler)
    }

    /// Add handler for TRACE type requests.
    #[must_use]
    pub fn trace(self, handler: Handler<ReqBody, ResBody>) -> Self {
        self.using(Method::TRACE, handler)
    }

    /// Add handler for CONNECT type requests.
    #[must_use]
    pub fn connect(self, handler: Handler<ReqBody, ResBody>) -> Self {
        self.using(Method::CONNECT, handler)
    }

    /// Add handler for PATCH type requests.
    #[must_use]
    pub fn patch(self, handler: Handler<ReqBody, ResBody>) -> Self {
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

    fn using(mut self, method: Method, handler: Handler<ReqBody, ResBody>) -> Self {
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
