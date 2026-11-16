use super::{
    route::Route,
    router::{ErrorHandler, Router},
};
use http::{Response, StatusCode};
use prefix_tree_map::{KeyPart, PrefixTreeMapBuilder};
use std::sync::Arc;

/// HTTP Router builder
/// Responsible for building a HTTP Router based on defined Route
pub struct RouterBuilder<ReqBody, ResBody> {
    routes: PrefixTreeMapBuilder<String, String, Route<ReqBody, ResBody>>,
    error_handler: Option<ErrorHandler<ResBody>>,
}

impl<ReqBody, ResBody> RouterBuilder<ReqBody, ResBody> {
    /// Create new `RouterBuilder`
    #[must_use]
    pub fn new() -> Self {
        Self {
            routes: PrefixTreeMapBuilder::new(),
            error_handler: None,
        }
    }

    /// Add a Route to the Router
    #[must_use]
    pub fn route(mut self, route: Route<ReqBody, ResBody>) -> Self {
        self.routes.insert(Self::build_path(&route.path), route);
        self
    }

    /// Replace the built-in `404 Not Found` / `405 Method Not Allowed` responses.
    ///
    /// The handler gets the status code and returns the full response. For a 405 the
    /// router adds an `Allow` header listing the path's methods, unless the handler
    /// already set one.
    #[must_use]
    pub fn error_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(StatusCode) -> Response<ResBody> + Send + Sync + 'static,
    {
        self.error_handler = Some(Arc::new(handler));
        self
    }

    fn build_path(path: &str) -> Vec<KeyPart<String, String>> {
        path.split('/')
            .map(|part| {
                if part.starts_with(':') {
                    KeyPart::Wildcard(part.to_string())
                } else {
                    KeyPart::Exact(part.to_string())
                }
            })
            .collect()
    }

    /// Build the router service for the routes
    #[must_use]
    pub fn build(self) -> Router<ReqBody, ResBody> {
        let mut router = Router::new(self.routes.build());
        if let Some(handler) = self.error_handler {
            router = router.with_error_handler(handler);
        }
        router
    }
}

impl<ReqBody, ResBody> Default for RouterBuilder<ReqBody, ResBody> {
    fn default() -> Self {
        Self::new()
    }
}
