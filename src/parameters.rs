use prefix_tree_map::Captures;
use std::{borrow::Cow, collections::HashMap};

/// URL parameters passed to handlers.
///
/// A route segment written as `:user_id` (or a trailing catch-all `*rest`) is stored
/// under its bare name, so look it up with `params.get("user_id")`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UrlParams {
    /// Values captured from the URL, keyed by parameter name without the `:`/`*` prefix.
    pub captures: HashMap<String, String>,
}

impl UrlParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a captured parameter by name, e.g. `params.get("user_id")`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.captures.get(name).map(String::as_str)
    }
}

impl Captures<String, Cow<'_, str>> for UrlParams {
    fn insert(&mut self, key: String, value: Cow<'_, str>) {
        self.captures.insert(key, value.into_owned());
    }
}
