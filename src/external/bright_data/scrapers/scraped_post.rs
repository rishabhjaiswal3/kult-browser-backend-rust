// src/external/bright_data/scrapers/scraped_post.rs
//
// Normalized post data extracted from raw BrightData JSON.
// Platform-specific field names are resolved here — callers
// never need to know that TikTok uses "digg_count" vs "likes".

/// Normalized post data returned by the BrightData scraper.
///
/// All platform-specific field names have been mapped to unified fields.
/// The `social_media` module only sees this struct — never raw JSON.
#[derive(Debug, Clone)]
pub struct ScrapedPostData {
    /// Hashtags from the post, lowercased, without leading '#'
    /// e.g. ["gamedev", "indiedev", "kultgames"]
    pub hashtags: Vec<String>,

    /// External URLs from dedicated URL fields (not from text)
    /// e.g. Twitter's `external_url`, Facebook's `post_external_link`, LinkedIn/Reddit's `embedded_links`
    pub external_urls: Vec<String>,

    /// The full text content of the post (description, content, post_text, or title depending on platform)
    pub text_content: String,

    /// Engagement count: likes, digg_count, num_upvotes, num_likes depending on platform
    pub likes: u32,

    /// BrightData error field — if set, the post is dead/removed/unavailable
    pub error: Option<String>,

    /// Full raw BrightData JSON response for debugging/storage
    pub raw: serde_json::Value,
}
