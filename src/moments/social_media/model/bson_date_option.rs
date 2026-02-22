use chrono::{DateTime, Utc};
use mongodb::bson::DateTime as BsonDateTime;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(dt) => {
            let bson_dt = BsonDateTime::from_millis(dt.timestamp_millis());
            bson_dt.serialize(serializer)
        }
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let maybe_bson_dt: Option<BsonDateTime> = Option::deserialize(deserializer)?;
    match maybe_bson_dt {
        Some(bson_dt) => Ok(Some(bson_dt.to_chrono())),
        None => Ok(None),
    }
}
