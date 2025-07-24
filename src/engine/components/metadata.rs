#[derive(Clone, Debug)]
pub struct Metadata {
    title: String,
}

impl Metadata {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}
