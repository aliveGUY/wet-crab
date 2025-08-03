use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    pub title: String,
    pub role: Option<String>, // Entity role for global variable binding
}

impl Metadata {
    pub fn new(title: &str) -> Self {
        Self::new_with_role(title, None)
    }

    pub fn new_with_role(title: &str, role: Option<&str>) -> Self {
        Self {
            title: title.to_string(),
            role: role.map(|r| r.to_string()),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
}
