use chrono::{DateTime, Utc};
use mongodb::bson::{self, oid::ObjectId, Bson, Document};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

use super::game_image_model::GameImages;

fn default_platform() -> String {
    "web".to_string()
}

fn serialize_optional_datetime<S>(
    value: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => bson::DateTime::from_millis(value.timestamp_millis()).serialize(serializer),
        None => serializer.serialize_none(),
    }
}

fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Bson>::deserialize(deserializer)?;

    Ok(match value {
        None | Some(Bson::Null) => None,
        Some(Bson::DateTime(value)) => Some(value.to_chrono()),
        Some(Bson::String(value)) => parse_datetime_string(&value),
        Some(Bson::Int64(value)) => DateTime::<Utc>::from_timestamp_millis(value),
        Some(Bson::Int32(value)) => DateTime::<Utc>::from_timestamp_millis(value as i64),
        Some(Bson::Document(value)) => parse_datetime_document(&value),
        _ => None,
    })
}

fn parse_datetime_document(document: &Document) -> Option<DateTime<Utc>> {
    let value = document.get("$date")?;
    parse_datetime_bson(value)
}

fn parse_datetime_bson(value: &Bson) -> Option<DateTime<Utc>> {
    match value {
        Bson::DateTime(value) => Some(value.to_chrono()),
        Bson::String(value) => parse_datetime_string(value),
        Bson::Int64(value) => DateTime::<Utc>::from_timestamp_millis(*value),
        Bson::Int32(value) => DateTime::<Utc>::from_timestamp_millis(*value as i64),
        Bson::Document(value) => value
            .get_str("$numberLong")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .and_then(DateTime::<Utc>::from_timestamp_millis),
        _ => None,
    }
}

fn parse_datetime_string(value: &str) -> Option<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

// Main game entity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GameModel {
    // Essential
    #[serde(rename = "_id")]
    #[schema(value_type = String)]
    pub id: ObjectId,
    pub identification: String, // Unique slug (game name with lowercase and hyphens)
    pub name: super::Localized<String>,
    #[serde(alias = "type", default = "default_platform")]
    pub platform: String, // "web", "desktop", "mobile"
    #[serde(default)]
    pub url: String,
    pub images: GameImages,
    #[serde(rename = "isReleased", alias = "is_released", default)]
    pub is_released: bool,

    // Optional
    pub slogan: Option<super::Localized<String>>,
    #[schema(value_type = Option<Object>)]
    pub about: Option<serde_json::Value>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub rating: Option<f64>,
    pub rating_count: Option<u32>,

    // Extra
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<serde_json::Value>,

    // Timestamps
    #[serde(
        alias = "createdAt",
        default,
        deserialize_with = "deserialize_optional_datetime",
        serialize_with = "serialize_optional_datetime"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(
        alias = "updatedAt",
        default,
        deserialize_with = "deserialize_optional_datetime",
        serialize_with = "serialize_optional_datetime"
    )]
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::GameModel;
    use crate::game::model::{util, GameImages};
    use mongodb::bson::{doc, oid::ObjectId, to_bson};

    fn base_game_doc() -> mongodb::bson::Document {
        doc! {
            "_id": ObjectId::new(),
            "identification": "test-game",
            "name": to_bson(&util::create_localized("Test Game".to_string())).unwrap(),
            "platform": "web",
            "url": "https://example.com/game",
            "images": to_bson(&GameImages::default()).unwrap(),
            "isReleased": true,
        }
    }

    #[test]
    fn deserializes_string_and_extended_json_timestamps() {
        let mut doc = base_game_doc();
        doc.insert("createdAt", "2026-03-24T10:15:30Z");
        doc.insert("updatedAt", doc! { "$date": "2026-03-24T10:16:30Z" });

        let game: GameModel = mongodb::bson::from_document(doc).expect("game should deserialize");

        assert!(game.created_at.is_some());
        assert!(game.updated_at.is_some());
    }

    #[test]
    fn ignores_invalid_timestamp_shapes_instead_of_failing() {
        let mut doc = base_game_doc();
        doc.insert("createdAt", "not-a-date");
        doc.insert(
            "updatedAt",
            doc! { "$date": { "$numberLong": "1711275390000" } },
        );

        let game: GameModel = mongodb::bson::from_document(doc).expect("game should deserialize");

        assert!(game.created_at.is_none());
        assert!(game.updated_at.is_some());
    }
}
