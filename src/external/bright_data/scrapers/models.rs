// src/external/bright_data/scrapers/models.rs
//
// Per-platform DTOs returned to callers.
// Each struct deserializes directly from Bright Data's JSON response.
// Field names match BD's API output — no translation layer needed.
//
// Note: BD sometimes returns numbers as strings (e.g. "94" for comment_count).
// We use a custom deserializer to handle both string and number formats.

use serde::{Deserialize, Deserializer, Serialize};

/// Deserializes a value that can be either a number or a string containing a number.
/// BD's API is inconsistent — some fields come as `94` and others as `"94"`.
fn string_or_u64<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
    let v: Option<serde_json::Value> = Option::deserialize(d)?;
    Ok(v.and_then(|v| match v {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => s.replace(',', "").parse().ok(),
        _ => None,
    }))
}

// ─── Twitter / X ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterPost {
    pub url: Option<String>,
    pub description: Option<String>,
    pub user_posted: Option<String>,
    pub name: Option<String>,
    pub date_posted: Option<String>,
    pub photos: Option<Vec<String>>,
    pub external_url: Option<String>,
    pub hashtags: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub likes: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub replies: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub reposts: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub views: Option<u64>,
    pub error: Option<String>,

    /// Catch-all for any extra fields BD returns that we haven't mapped explicitly
    #[serde(flatten)]
    pub raw_data: std::collections::HashMap<String, serde_json::Value>,
}

// ─── Instagram ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramPost {
    pub url: Option<String>,
    pub user_posted: Option<String>,
    pub description: Option<String>,
    pub hashtags: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub likes: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub num_comments: Option<u64>,
    pub date_posted: Option<String>,
    pub photos: Option<Vec<String>>,
    pub alt_text: Option<String>,
    pub error: Option<String>,

    /// Catch-all for any extra fields BD returns that we haven't mapped explicitly
    #[serde(flatten)]
    pub raw_data: std::collections::HashMap<String, serde_json::Value>,
}

// ─── TikTok ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokPost {
    pub url: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub profile_username: Option<String>,
    pub hashtags: Option<Vec<String>>,
    /// TikTok calls likes "digg_count"
    #[serde(default, deserialize_with = "string_or_u64")]
    pub digg_count: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub collect_count: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub comment_count: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub share_count: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub play_count: Option<u64>,
    pub date_posted: Option<String>,
    pub create_time: Option<String>,
    pub video_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub error: Option<String>,

    /// Catch-all for any extra fields BD returns that we haven't mapped explicitly
    #[serde(flatten)]
    pub raw_data: std::collections::HashMap<String, serde_json::Value>,
}

// ─── Facebook ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookPost {
    pub content: Option<String>,
    pub profile_handle: Option<String>,
    pub hashtags: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub likes: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub num_comments: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub shares: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub video_view_count: Option<u64>,
    pub date_posted: Option<String>,
    pub post_image: Option<String>,
    pub post_type: Option<String>,
    pub post_external_link: Option<String>,
    pub error: Option<String>,

    /// Catch-all for any extra fields BD returns that we haven't mapped explicitly
    #[serde(flatten)]
    pub raw_data: std::collections::HashMap<String, serde_json::Value>,
}

// ─── Reddit ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditPost {
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub user_posted: Option<String>,
    pub community_name: Option<String>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub num_upvotes: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub num_comments: Option<u64>,
    pub date_posted: Option<String>,
    pub photos: Option<Vec<String>>,
    pub tag: Option<String>,
    pub error: Option<String>,

    /// Catch-all for any extra fields BD returns that we haven't mapped explicitly
    #[serde(flatten)]
    pub raw_data: std::collections::HashMap<String, serde_json::Value>,
}

// ─── LinkedIn ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedInPost {
    pub post_url: Option<String>,
    pub post_text: Option<String>,
    pub user_id: Option<String>,
    pub headline: Option<String>,
    pub hashtags: Option<Vec<String>>,
    pub embedded_links: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub num_likes: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub num_comments: Option<u64>,
    pub date_posted: Option<String>,
    pub images: Option<Vec<String>>,
    pub error: Option<String>,

    /// Catch-all for any extra fields BD returns that we haven't mapped explicitly
    #[serde(flatten)]
    pub raw_data: std::collections::HashMap<String, serde_json::Value>,
}

// ─── Pinterest ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinterestPost {
    pub url: Option<String>,
    pub post_id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub date_posted: Option<String>,
    pub post_type: Option<String>,
    pub user_name: Option<String>,
    pub user_url: Option<String>,
    pub user_id: Option<String>,

    #[serde(default, deserialize_with = "string_or_u64")]
    pub followers: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub likes: Option<u64>,
    #[serde(default, deserialize_with = "string_or_u64")]
    pub comments_num: Option<u64>,

    pub categories: Option<Vec<String>>,
    pub image_video_url: Option<String>,
    pub attached_files: Option<Vec<String>>,
    pub hashtags: Option<Vec<String>>,
    pub error: Option<String>,

    /// Catch-all for any extra fields BD returns that we haven't mapped explicitly
    #[serde(flatten)]
    pub raw_data: std::collections::HashMap<String, serde_json::Value>,
}
