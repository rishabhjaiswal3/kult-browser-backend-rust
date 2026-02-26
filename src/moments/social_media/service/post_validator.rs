// src/moments/social_media/service/post_validator.rs
//
// Kult-specific post validation.
// Checks whether a scraped post is about Kult using three methods:
// 1. Hashtag match (#kultgames)
// 2. URL match in dedicated fields (kult.games)
// 3. Regex on text content (kult.games URL or @kultgames mention)

use crate::external::bright_data::scrapers::scraped_post::ScrapedPostData;

/// Kult post validator.
///
/// Operates on `ScrapedPostData` — never touches raw JSON or
/// platform-specific field names. All that is handled upstream
/// by the BrightData normalizer.
pub struct PostValidator;

/// Why a post was validated as valid or invalid.
#[derive(Debug, Clone)]
pub enum ValidationReason {
    /// Matched via hashtag (e.g. "kultgames")
    Hashtag(String),
    /// Matched via dedicated URL field (e.g. "https://kult.games/...")
    UrlField(String),
    /// Matched via regex on text content
    TextRegex(String),
    /// No Kult signal found
    NoMatch,
    /// Post returned an error from BrightData (dead/removed)
    ScraperError(String),
}

impl std::fmt::Display for ValidationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationReason::Hashtag(h) => write!(f, "hashtag:{}", h),
            ValidationReason::UrlField(u) => write!(f, "url:{}", u),
            ValidationReason::TextRegex(m) => write!(f, "text:{}", m),
            ValidationReason::NoMatch => write!(f, "no_match"),
            ValidationReason::ScraperError(e) => write!(f, "error:{}", e),
        }
    }
}

impl PostValidator {
    /// Validate whether a scraped post is about Kult.
    ///
    /// Returns `(is_valid, reason)`.
    /// A post is valid if ANY of the three methods matches.
    pub fn validate(post: &ScrapedPostData) -> (bool, ValidationReason) {
        // Check for BD error first
        if let Some(ref err) = post.error {
            return (false, ValidationReason::ScraperError(err.clone()));
        }

        // Method 1: Hashtag match
        if let Some(reason) = Self::check_hashtags(post) {
            return (true, reason);
        }

        // Method 2: URL match in dedicated fields
        if let Some(reason) = Self::check_url_fields(post) {
            return (true, reason);
        }

        // Method 3: Regex on text content
        if let Some(reason) = Self::check_text_content(post) {
            return (true, reason);
        }

        (false, ValidationReason::NoMatch)
    }

    /// Method 1: Check `hashtags[]` for "kultgames" or "kult.games" (already lowercased by normalizer)
    fn check_hashtags(post: &ScrapedPostData) -> Option<ValidationReason> {
        for tag in &post.hashtags {
            let tag_lower = tag.to_lowercase();
            if tag_lower == "kultgames" || tag_lower == "kult.games" || tag_lower == "kult" {
                return Some(ValidationReason::Hashtag(tag.clone()));
            }
        }
        None
    }

    /// Method 2: Check `external_urls[]` for "kult.games"
    fn check_url_fields(post: &ScrapedPostData) -> Option<ValidationReason> {
        for url in &post.external_urls {
            if url.contains("kult.games") {
                return Some(ValidationReason::UrlField(url.clone()));
            }
        }
        None
    }

    /// Method 3: Regex on `text_content` for kult.games URL or @kultgames mention
    fn check_text_content(post: &ScrapedPostData) -> Option<ValidationReason> {
        let text = post.text_content.to_lowercase();

        // Check for kult.games URL
        if text.contains("kult.games") {
            return Some(ValidationReason::TextRegex("kult.games".to_string()));
        }

        // Check for @kultgames mention
        if text.contains("@kultgames") {
            return Some(ValidationReason::TextRegex("@kultgames".to_string()));
        }

        None
    }
}
