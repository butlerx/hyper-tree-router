use super::{route::Route, router::Router};
use prefix_tree_map::{KeyPart, PrefixTreeMapBuilder};

/// HTTP Router builder
/// Responsibel for building a HTTP Router based on defined Route
pub struct RouterBuilder {
    routes: PrefixTreeMapBuilder<String, String, Route>,
}

impl RouterBuilder {
    /// Create new RouterBuilder
    pub fn new() -> Self {
        RouterBuilder {
            routes: PrefixTreeMapBuilder::<String, String, Route>::new(),
        }
    }

    /// Add a Route to the Router
    pub fn route(mut self, route: Route) -> Self {
        self.routes
            .insert(RouterBuilder::build_path(route.path.clone()), route);
        self
    }

    fn build_path(path: String) -> Vec<KeyPart<String, String>> {
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

    /// Build the hyper service for the routes
    pub fn build(self) -> Router {
        Router::new(self.routes.build())
    }
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
