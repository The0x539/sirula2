use std::collections::HashMap;

#[derive(Default)]
pub struct History {
    pub last_used: HashMap<String, u64>,
}
