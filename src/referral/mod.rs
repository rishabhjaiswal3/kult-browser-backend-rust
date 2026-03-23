// src/referral/mod.rs

pub mod analytics;
pub mod anti_fraud;
pub mod model;
pub mod redirect_route;
pub mod route;
pub mod service;
pub mod worker;

pub use crate::redis::keys::REFERRAL_CLICK_QUEUE as CLICK_QUEUE;
pub use crate::redis::keys::REFERRAL_VERIFY_QUEUE as VERIFY_QUEUE;
