use super::{
    route::Route,
    router::{CatchAll, ErrorHandler, Router},
};
use http::{Response, StatusCode};
use prefix_tree_map::{KeyPart, PrefixTreeMapBuilder};
use std::{borrow::Cow, sync::Arc};

type TreeBuilder<V> = PrefixTreeMapBuilder<Cow<'static, str>, String, V>;

/// HTTP Router builder. Start with `RouterBuilder::default()`, add routes, then `build()`.
pub struct RouterBuilder<ReqBody, ResBody> {
    routes: TreeBuilder<Route<ReqBody, ResBody>>,
    catch_all: TreeBuilder<CatchAll<ReqBody, ResBody>>,
    catch_all_depths: Vec<usize>,
    error_handler: Option<ErrorHandler<ResBody>>,
}

impl<ReqBody, ResBody> RouterBuilder<ReqBody, ResBody> {
    /// Add a Route to the Router.
    ///
    /// Path segments starting with `:` capture one segment (`/user/:id`). A final
    /// segment starting with `*` captures the rest of the path, one or more segments
    /// joined with `/` (`/files/*path`). Exact and `:param` routes take priority over
    /// catch-alls.
    ///
    /// # Panics
    ///
    /// If a `*` segment is not the last segment of the path.
    #[must_use]
    pub fn route(mut self, route: Route<ReqBody, ResBody>) -> Self {
        let mut segments: Vec<&str> = route.path.split('/').collect();
        let catch_all = match segments.last() {
            Some(last) if last.starts_with('*') => {
                let name = last[1..].to_string();
                segments.pop();
                Some(name)
            }
            _ => None,
        };
        assert!(
            !segments.iter().any(|s| s.starts_with('*')),
            "catch-all `*` must be the last segment: {}",
            route.path
        );
        let key = Self::build_path(&segments);
        match catch_all {
            Some(name) => {
                self.catch_all_depths.push(key.len());
                self.catch_all.insert(key, CatchAll { name, route });
            }
            None => self.routes.insert(key, route),
        }
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

    fn build_path(segments: &[&str]) -> Vec<KeyPart<Cow<'static, str>, String>> {
        segments
            .iter()
            .map(|part| match part.strip_prefix(':') {
                Some(name) => KeyPart::Wildcard(name.to_string()),
                None => KeyPart::Exact(Cow::Owned((*part).to_string())),
            })
            .collect()
    }

    /// Build the router service for the routes
    #[must_use]
    pub fn build(self) -> Router<ReqBody, ResBody> {
        let mut router = Router::new(
            self.routes.build(),
            self.catch_all.build(),
            self.catch_all_depths,
        );
        if let Some(handler) = self.error_handler {
            router = router.with_error_handler(handler);
        }
        router
    }
}

// Manual impl: `#[derive(Default)]` would wrongly require `ReqBody: Default + ResBody: Default`.
impl<ReqBody, ResBody> Default for RouterBuilder<ReqBody, ResBody> {
    fn default() -> Self {
        Self {
            routes: PrefixTreeMapBuilder::default(),
            catch_all: PrefixTreeMapBuilder::default(),
            catch_all_depths: Vec::new(),
            error_handler: None,
        }
    }
}
