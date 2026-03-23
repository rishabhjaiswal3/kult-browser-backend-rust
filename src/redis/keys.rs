use once_cell::sync::Lazy;

use crate::config::CONFIG;

pub fn prefixed(parts: &[&str]) -> String {
    let mut segments = Vec::with_capacity(parts.len() + 1);
    segments.push(CONFIG.valkey.key_prefix.as_str());
    segments.extend(parts.iter().copied());
    segments.join(":")
}

fn processing_key(queue_name: &str) -> String {
    format!("{queue_name}:processing")
}

fn queue_key(parts: &[&str]) -> String {
    let mut queue_parts = Vec::with_capacity(parts.len() + 1);
    queue_parts.extend(parts.iter().copied());
    queue_parts.push("queue");
    prefixed(&queue_parts)
}

fn dead_letter_key(queue_name: &str) -> String {
    format!("{queue_name}:dead_letter")
}

pub static MOMENTS_ZERO_G_MIGRATION_QUEUE: Lazy<String> =
    Lazy::new(|| queue_key(&["moments", "zero_g", "migration"]));
pub static MOMENTS_ZERO_G_MIGRATION_PROCESSING_QUEUE: Lazy<String> =
    Lazy::new(|| processing_key(MOMENTS_ZERO_G_MIGRATION_QUEUE.as_str()));
pub static MOMENTS_ZERO_G_MIGRATION_DEAD_LETTER_QUEUE: Lazy<String> =
    Lazy::new(|| dead_letter_key(MOMENTS_ZERO_G_MIGRATION_QUEUE.as_str()));
pub static MOMENTS_ZERO_G_MIGRATION_DEAD_LETTER_PROCESSING_QUEUE: Lazy<String> =
    Lazy::new(|| processing_key(MOMENTS_ZERO_G_MIGRATION_DEAD_LETTER_QUEUE.as_str()));

pub static MOMENTS_BRIGHT_DATA_POST_SCRAPE_QUEUE: Lazy<String> =
    Lazy::new(|| queue_key(&["moments", "bright_data", "post_scrape"]));
pub static MOMENTS_BRIGHT_DATA_POST_SCRAPE_PROCESSING_QUEUE: Lazy<String> =
    Lazy::new(|| processing_key(MOMENTS_BRIGHT_DATA_POST_SCRAPE_QUEUE.as_str()));
pub static MOMENTS_BRIGHT_DATA_POST_SCRAPE_DEAD_LETTER_QUEUE: Lazy<String> =
    Lazy::new(|| dead_letter_key(MOMENTS_BRIGHT_DATA_POST_SCRAPE_QUEUE.as_str()));
pub static MOMENTS_BRIGHT_DATA_POST_SCRAPE_DEAD_LETTER_PROCESSING_QUEUE: Lazy<String> =
    Lazy::new(|| processing_key(MOMENTS_BRIGHT_DATA_POST_SCRAPE_DEAD_LETTER_QUEUE.as_str()));

pub static REFERRAL_CLICK_QUEUE: Lazy<String> =
    Lazy::new(|| queue_key(&["referral", "click"]));
pub static REFERRAL_CLICK_PROCESSING_QUEUE: Lazy<String> =
    Lazy::new(|| processing_key(REFERRAL_CLICK_QUEUE.as_str()));

pub static REFERRAL_VERIFY_QUEUE: Lazy<String> =
    Lazy::new(|| queue_key(&["referral", "verification"]));
pub static REFERRAL_VERIFY_PROCESSING_QUEUE: Lazy<String> =
    Lazy::new(|| processing_key(REFERRAL_VERIFY_QUEUE.as_str()));

pub fn referral_code_cache_key(code: &str) -> String {
    prefixed(&["referral", "code_cache", code])
}

pub fn referral_signup_ip_key(ip_hash: &str) -> String {
    prefixed(&["referral", "anti_fraud", "signup_ip", ip_hash])
}
