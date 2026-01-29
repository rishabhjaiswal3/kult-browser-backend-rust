use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Localized Wrapper ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Localized<T> {
    pub default: String,
    #[serde(flatten)]
    pub variants: HashMap<String, T>,
}

impl<T> Default for Localized<T> {
    fn default() -> Self {
        Self {
            default: "en".to_string(),
            variants: HashMap::new(),
        }
    }
}
// PartialEq for testing
impl<T: PartialEq> PartialEq for Localized<T> {
    fn eq(&self, other: &Self) -> bool {
        self.default == other.default && self.variants == other.variants
    }
}
