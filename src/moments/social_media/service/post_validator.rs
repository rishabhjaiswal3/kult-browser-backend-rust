// src/moments/social_media/service/post_validator.rs
//
// Kult-specific post validation.
// Checks whether a scraped post matches the configured validation terms using
// three methods:
// 1. Hashtag match
// 2. URL match in dedicated fields
// 3. Text-content match
//
// The keywords themselves are declared in `CONFIG.scrape.validation_terms`,
// loaded from `SCRAPE_VALIDATION_TERMS` in `src/config/scrape_config.rs`.

use crate::config::CONFIG;
use crate::external::bright_data::scrapers::scraped_post::ScrapedPostData;
use reqwest::Url;

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

    fn validation_terms() -> &'static [String] {
        &CONFIG.scrape.validation_terms
    }

    /// Method 1: Check `hashtags[]` using the configured validation terms.
    ///
    /// These terms are declared in `CONFIG.scrape.validation_terms`.
    fn check_hashtags(post: &ScrapedPostData) -> Option<ValidationReason> {
        for tag in &post.hashtags {
            let tag_normalized = Self::normalize_compact(tag);
            for term in Self::validation_terms() {
                if !tag_normalized.is_empty()
                    && tag_normalized == Self::normalize_compact(term)
                {
                    return Some(ValidationReason::Hashtag(tag.clone()));
                }
            }
        }
        None
    }

    /// Method 2: Check `external_urls[]` using the configured validation terms.
    ///
    /// These terms are declared in `CONFIG.scrape.validation_terms`.
    fn check_url_fields(post: &ScrapedPostData) -> Option<ValidationReason> {
        for url in &post.external_urls {
            for term in Self::validation_terms() {
                if Self::url_matches_term(url, term) {
                    return Some(ValidationReason::UrlField(url.clone()));
                }
            }
        }
        None
    }

    /// Method 3: Check `text_content` using the configured validation terms.
    ///
    /// These terms are declared in `CONFIG.scrape.validation_terms`.
    fn check_text_content(post: &ScrapedPostData) -> Option<ValidationReason> {
        let text = post.text_content.to_lowercase();

        for term in Self::validation_terms() {
            if Self::contains_bounded_term(&text, &term.to_lowercase()) {
                return Some(ValidationReason::TextRegex(term.clone()));
            }
        }

        None
    }

    fn normalize_compact(value: &str) -> String {
        value
            .chars()
            .filter(|ch| ch.is_alphanumeric())
            .flat_map(|ch| ch.to_lowercase())
            .collect()
    }

    fn url_matches_term(url: &str, term: &str) -> bool {
        let normalized_term = Self::normalize_compact(term);
        if normalized_term.is_empty() {
            return false;
        }

        let parsed = match Url::parse(url) {
            Ok(parsed) => parsed,
            Err(_) => return Self::contains_bounded_term(&url.to_lowercase(), &term.to_lowercase()),
        };

        if let Some(host) = parsed.host_str() {
            let host_lower = host.to_lowercase();
            if Self::normalize_compact(&host_lower) == normalized_term {
                return true;
            }

            for label in host_lower.split('.') {
                if Self::normalize_compact(label) == normalized_term {
                    return true;
                }
            }
        }

        for segment in parsed.path_segments().into_iter().flatten() {
            if Self::normalize_compact(segment) == normalized_term {
                return true;
            }
        }

        if let Some(query) = parsed.query() {
            for piece in query.split(&['&', '='][..]) {
                if Self::normalize_compact(piece) == normalized_term {
                    return true;
                }
            }
        }

        false
    }

    fn contains_bounded_term(text: &str, term: &str) -> bool {
        if term.is_empty() {
            return false;
        }

        for (idx, _) in text.match_indices(term) {
            let before = text[..idx].chars().next_back();
            let after = text[idx + term.len()..].chars().next();

            let before_ok = before.is_none_or(|ch| !ch.is_alphanumeric());
            let after_ok = after.is_none_or(|ch| !ch.is_alphanumeric());

            if before_ok && after_ok {
                return true;
            }
        }

        false
    }
}
