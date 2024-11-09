pub struct SetArguments {
    pub key: String,
    pub value: String,
    pub px: Option<u128>,
}

impl SetArguments {
    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value,
            px: None,
        }
    }
}
