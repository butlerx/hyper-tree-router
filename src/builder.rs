use super::{route::Route, router::Router};
use prefix_tree_map::{KeyPart, PrefixTreeMapBuilder};

/// HTTP Router builder
/// Responsible for building a HTTP Router based on defined Route
pub struct RouterBuilder<ReqBody, ResBody> {
    routes: PrefixTreeMapBuilder<String, String, Route<ReqBody, ResBody>>,
}

impl<ReqBody, ResBody> RouterBuilder<ReqBody, ResBody> {
    /// Create new `RouterBuilder`
    #[must_use]
    pub fn new() -> Self {
        Self {
            routes: PrefixTreeMapBuilder::new(),
        }
    }

    /// Add a Route to the Router
    #[must_use]
    pub fn route(mut self, route: Route<ReqBody, ResBody>) -> Self {
        self.routes.insert(Self::build_path(&route.path), route);
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
        Router::new(self.routes.build())
    }
}

impl<ReqBody, ResBody> Default for RouterBuilder<ReqBody, ResBody> {
    fn default() -> Self {
        Self::new()
    }
}
