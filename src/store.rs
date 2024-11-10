use std::{collections::HashMap, path::PathBuf};

pub struct Store {
    cache: HashMap<String, Value>,
    config: Config,
}

impl Store {
    pub fn new() -> Self {
        let cache = HashMap::new();

        Self {
            cache,
            config: Config::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String, px: Option<u128>) {
        let mut value = Value::new(value);

        if let Some(px) = px {
            value.with_px(px);
        }

        self.cache.insert(key, value);
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        let value = self.cache.get(key);

        let value = value.map(|value| {
            if value.has_expired() {
                None
            } else {
                Some(&value.value)
            }
        });

        value.flatten()
    }

    pub fn clean_expired_keys(&mut self) {
        self.cache.retain(|_, value| !value.has_expired());
    }

    pub fn set_rdb_dir(&mut self, rdb_dir: PathBuf) {
        self.config.rdb_dir = Some(rdb_dir);
    }

    pub fn set_rdb_file_name(&mut self, rdb_file_name: String) {
        self.config.rdb_file_name = Some(rdb_file_name);
    }

    pub fn get_config(&self, key: &str) -> Option<&str> {
        match key {
            "dir" => self
                .config
                .rdb_dir
                .as_ref()
                .and_then(|rdb_dir| rdb_dir.to_str()),
            "dbfilename" => self.config.rdb_file_name.as_deref(),
            _ => None,
        }
    }
}

struct Value {
    value: String,
    px: Option<Px>,
}

impl Value {
    fn new(value: String) -> Self {
        Self { value, px: None }
    }

    fn with_px(&mut self, miliseconds: u128) {
        let px = Px::new(miliseconds);

        self.px = Some(px);
    }

    fn has_expired(&self) -> bool {
        self.px.as_ref().map_or(false, |px| px.has_expired())
    }
}

struct Px {
    instant: std::time::Instant,
    miliseconds: u128,
}

impl Px {
    fn new(miliseconds: u128) -> Self {
        let instant = std::time::Instant::now();

        Self {
            instant,
            miliseconds,
        }
    }

    fn has_expired(&self) -> bool {
        self.instant.elapsed().as_millis() > self.miliseconds
    }
}

struct Config {
    rdb_dir: Option<PathBuf>,
    rdb_file_name: Option<String>,
}

impl Config {
    fn new() -> Self {
        Self {
            rdb_dir: None,
            rdb_file_name: None,
        }
    }
}
