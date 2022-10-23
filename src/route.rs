use super::parameters;
use hyper::{Body, Method, Request, Response};
use std::{collections::HashMap, fmt};

pub type Handler = fn(parameters::UrlParams, Request<Body>) -> Response<Body>;

// Holds route information
#[derive(Clone)]
pub struct Route {
    /// Path to match
    pub path: String,

    /// Request handlers
    pub handlers: HashMap<Method, Handler>,
}

impl Route {
    /// Add handler for OPTIONS type requests
    pub fn options(self, handler: Handler) -> Self {
        self.using(Method::OPTIONS, handler)
    }

    /// Add handler for GET type requests
    pub fn get(self, handler: Handler) -> Self {
        self.using(Method::GET, handler)
    }

    /// Add handler for POST type requests
    pub fn post(self, handler: Handler) -> Self {
        self.using(Method::POST, handler)
    }

    /// Add handler for PUT type requests
    pub fn put(self, handler: Handler) -> Self {
        self.using(Method::PUT, handler)
    }

    /// Add handler for DELETE type requests
    pub fn delete(self, handler: Handler) -> Self {
        self.using(Method::DELETE, handler)
    }

    /// Add handler for HEAD type requests
    pub fn head(self, handler: Handler) -> Self {
        self.using(Method::HEAD, handler)
    }

    /// Add handler for TRACE type requests
    pub fn trace(self, handler: Handler) -> Self {
        self.using(Method::TRACE, handler)
    }

    /// Add handler for CONNECT type requests
    pub fn connect(self, handler: Handler) -> Self {
        self.using(Method::CONNECT, handler)
    }

    /// Add handler for PATCH type requests
    pub fn patch(self, handler: Handler) -> Self {
        self.using(Method::PATCH, handler)
    }

    /// Create `Route` for a given url
    pub fn url(path: &str) -> Self {
        Self {
            path: path.to_string(),
            ..Self::default()
        }
    }

    fn using(mut self, method: Method, handler: Handler) -> Self {
        self.handlers.insert(method, handler);
        self
    }
}

impl Default for Route {
    fn default() -> Self {
        Self {
            path: "/".to_string(),
            handlers: HashMap::new(),
        }
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Route {{methods: {:?}, path: {:?}}}",
            self.handlers.keys(),
            self.path
        )
    }
}
