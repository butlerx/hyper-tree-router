use prefix_tree_map::Captures;
use std::collections::HashMap;

/// URL Paramaters to be passed to handlers.
/// Contains a HashMap containing parameters caputred from the url.
#[derive(Debug, Clone)]
pub struct UrlParams {
    /// HashMap Containing Programatic elements captured the URL.
    /// If the  Element was defined as `:user_id` the same key can be used to look it up
    pub captures: HashMap<String, String>,
}

impl UrlParams {
    pub fn new() -> Self {
        Self {
            captures: HashMap::new(),
        }
    }
}

impl Default for UrlParams {
    fn default() -> Self {
        Self::new()
    }
}

impl Captures<&str, &str> for UrlParams {
    fn insert(&mut self, key: &str, value: &str) {
        self.captures.insert(key.to_string(), value.to_string());
    }
}

impl Captures<String, String> for UrlParams {
    fn insert(&mut self, key: String, value: String) {
        self.captures.insert(key, value);
    }
}
