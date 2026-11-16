use prefix_tree_map::Captures;
use std::borrow::Cow;

/// URL parameters passed to handlers.
///
/// A route segment written as `:user_id` (or a trailing catch-all `*rest`) is stored
/// under its bare name, so look it up with `params.get("user_id")`.
///
/// Params are kept in path order in a small `Vec`: routes rarely have more than a
/// few, so a linear scan beats hashing, and repeated names keep every value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UrlParams {
    captures: Vec<(String, String)>,
}

impl UrlParams {
    /// Value of the first parameter called `name`, e.g. `params.get("user_id")`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.captures
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// All `(name, value)` pairs in the order they appear in the path.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.captures
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }

    /// Number of captured parameters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.captures.len()
    }

    /// Whether no parameters were captured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.captures.is_empty()
    }

    #[cfg_attr(not(feature = "serde"), allow(dead_code))]
    pub(crate) fn into_pairs(self) -> Vec<(String, String)> {
        self.captures
    }

    pub(crate) fn push(&mut self, name: String, value: String) {
        self.captures.push((name, value));
    }
}

impl Captures<String, Cow<'_, str>> for UrlParams {
    fn insert(&mut self, key: String, value: Cow<'_, str>) {
        self.push(key, value.into_owned());
    }
}
