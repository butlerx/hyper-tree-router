use super::parameters;
use hyper::{Body, Method, Request, Response};
use std::{collections::HashMap, fmt};

pub type Handler = fn(parameters::UrlParams, Request<Body>) -> Response<Body>;

/// Holds route information
pub struct Route {
    /// Path to match
    pub path: String,

    /// Request handlers
    pub handlers: HashMap<Method, Handler>,
}

impl Route {
    pub fn options(self, handler: Handler) -> Self {
        self.using(Method::OPTIONS, handler)
    }

    pub fn get(self, handler: Handler) -> Self {
        self.using(Method::GET, handler)
    }

    pub fn post(self, handler: Handler) -> Self {
        self.using(Method::POST, handler)
    }

    pub fn put(self, handler: Handler) -> Self {
        self.using(Method::PUT, handler)
    }

    pub fn delete(self, handler: Handler) -> Self {
        self.using(Method::DELETE, handler)
    }

    pub fn head(self, handler: Handler) -> Self {
        self.using(Method::HEAD, handler)
    }

    pub fn trace(self, handler: Handler) -> Self {
        self.using(Method::TRACE, handler)
    }

    pub fn connect(self, handler: Handler) -> Self {
        self.using(Method::CONNECT, handler)
    }

    pub fn patch(self, handler: Handler) -> Self {
        self.using(Method::PATCH, handler)
    }

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
