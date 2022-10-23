use prefix_tree_map::Captures;
use std::collections::HashMap;

pub struct UrlParams {
    pub captures: HashMap<String, String>,
}

impl UrlParams {
    pub fn new() -> Self {
        Self {
            captures: HashMap::new(),
        }
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
