use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub request_count: u64,
    pub active_users: HashSet<String>,
}
