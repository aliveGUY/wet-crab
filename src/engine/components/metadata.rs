use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    pub title: String,
    pub role: Option<String>, // Entity role for global variable binding
    pub is_persist: bool,     // Whether this entity should be saved to JSON
}

impl Metadata {
    pub fn new(title: &str, role: Option<&str>, is_persist: Option<bool>) -> Self {
        Self {
            title: title.to_string(),
            role: role.map(|r| r.to_string()),
            is_persist: is_persist.unwrap_or(true), // Default to persistent
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
}
