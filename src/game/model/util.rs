use super::localized_model::Localized;
use std::collections::HashMap;

// Helper to create a simple En-only localized value
pub fn create_localized<T: Clone>(val: T) -> Localized<T> {
    let mut variants = HashMap::new();
    variants.insert("en".to_string(), val);
    Localized {
        default: "en".to_string(),
        variants,
    }
}

// Helper to create localized value with English and Mock Chinese (zh)
pub fn create_localized_with_cn(val: String) -> Localized<String> {
    let mut variants = HashMap::new();
    variants.insert("en".to_string(), val.clone());
    variants.insert("zh".to_string(), format!("(CN) {}", val)); // Mock Chinese
    Localized {
        default: "en".to_string(),
        variants,
    }
}
