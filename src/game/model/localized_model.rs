use serde::{de::Deserializer, Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

// --- Localized Wrapper ---

#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(bound = "T: utoipa::ToSchema")]
pub struct Localized<T> {
    #[serde(default = "default_locale")]
    pub default: String,
    #[serde(flatten)]
    pub variants: HashMap<String, T>,
}

fn default_locale() -> String {
    "en".to_string()
}

impl<T> Default for Localized<T> {
    fn default() -> Self {
        Self {
            default: default_locale(),
            variants: HashMap::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
#[serde(untagged)]
enum LocalizedRepr<T> {
    Structured(LocalizedStructured<T>),
    Legacy(T),
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct LocalizedStructured<T> {
    #[serde(default = "default_locale")]
    default: String,
    #[serde(default, flatten)]
    variants: HashMap<String, T>,
}

impl<'de, T> Deserialize<'de> for Localized<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match LocalizedRepr::<T>::deserialize(deserializer)? {
            LocalizedRepr::Structured(value) => Ok(Self {
                default: value.default,
                variants: value.variants,
            }),
            LocalizedRepr::Legacy(value) => {
                let mut variants = HashMap::new();
                variants.insert(default_locale(), value);
                Ok(Self {
                    default: default_locale(),
                    variants,
                })
            }
        }
    }
}
// PartialEq for testing
impl<T: PartialEq> PartialEq for Localized<T> {
    fn eq(&self, other: &Self) -> bool {
        self.default == other.default && self.variants == other.variants
    }
}

#[cfg(test)]
mod tests {
    use super::Localized;
    use crate::game::model::{util, ImageObject};
    use bson::{doc, from_document};

    #[test]
    fn deserializes_legacy_string_bson_value() {
        let localized: Localized<String> = bson::from_bson(bson::Bson::String(
            "Formula Speed Thrills".to_string(),
        ))
        .expect("legacy string should deserialize");

        assert_eq!(
            localized,
            util::create_localized("Formula Speed Thrills".to_string())
        );
    }

    #[test]
    fn deserializes_structured_value_without_default() {
        let localized: Localized<String> = from_document(doc! {
            "en": "Formula Speed Thrills",
            "zh": "Formula Speed Thrills CN",
        })
        .expect("localized object without default should deserialize");

        assert_eq!(localized.default, "en");
        assert_eq!(
            localized.variants.get("en"),
            Some(&"Formula Speed Thrills".to_string())
        );
        assert_eq!(
            localized.variants.get("zh"),
            Some(&"Formula Speed Thrills CN".to_string())
        );
    }

    #[test]
    fn deserializes_legacy_image_object_as_en_localized_value() {
        let localized: Localized<ImageObject> = from_document(doc! {
            "url": "https://example.com/game-hero.png"
        })
        .expect("legacy image object should deserialize");

        assert_eq!(
            localized,
            util::create_localized(ImageObject {
                url: "https://example.com/game-hero.png".to_string(),
                ..Default::default()
            })
        );
    }
}
