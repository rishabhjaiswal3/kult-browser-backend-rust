// src/referral/mod.rs

pub mod analytics;
pub mod anti_fraud;
pub mod model;
pub mod redirect_route;
pub mod route;
pub mod service;
pub mod worker;

pub const CLICK_QUEUE: &str = "referrals:clicks";
pub const VERIFY_QUEUE: &str = "referrals:pending_checks";
