pub struct SetArguments {
    pub key: String,
    pub value: String,
}

impl SetArguments {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}
