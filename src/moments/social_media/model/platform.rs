use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Platform {
    Twitter,
    Farcaster,
    Pinterest,
}
