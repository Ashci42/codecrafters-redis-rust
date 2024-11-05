use std::collections::HashMap;

pub struct Store {
    cache: HashMap<String, String>,
}

impl Store {
    pub fn new() -> Self {
        let cache = HashMap::new();

        Self { cache }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.cache.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.cache.get(key)
    }
}
