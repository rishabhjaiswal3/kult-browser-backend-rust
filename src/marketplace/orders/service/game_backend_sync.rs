use serde_json::json;
use std::env;
use std::time::Duration;

use crate::marketplace::model::ListingModel;
use crate::marketplace::orders::model::OrderModel;

pub const DEFAULT_WARZONE_BACKEND_URL: &str = "https://api.warzonewarriors.xyz/warzone";
pub const DEFAULT_HIGHWAY_HUSTLE_BACKEND_URL: &str = "http://localhost:3400/api";

enum GameSyncTarget {
    Warzone,
    HighwayHustle,
    None,
}

impl GameSyncTarget {
    /// Factory: map gameIdentification to concrete sync target.
    fn from_game_id(game_id: &str) -> Self {
        match game_id.trim().to_lowercase().as_str() {
            "warzonewarriors" => Self::Warzone,
            "highwayhustle" => Self::HighwayHustle,
            _ => Self::None,
        }
    }
}

pub async fn sync_external_game_entitlement(
    order: &OrderModel,
    listing: &ListingModel,
    item_id: &str,
) -> Result<(), String> {
    let target = GameSyncTarget::from_game_id(&listing.game_identification);
    match target {
        GameSyncTarget::Warzone => sync_warzone_entitlement(order, listing, item_id).await,
        // Keep this explicit for future rollout. Right now, no direct HH write path is enabled.
        GameSyncTarget::HighwayHustle | GameSyncTarget::None => Ok(()),
    }
}

fn game_backend_url(target: &GameSyncTarget) -> String {
    match target {
        GameSyncTarget::Warzone => env::var("WARZONE_BACKEND_URL")
            .unwrap_or_else(|_| DEFAULT_WARZONE_BACKEND_URL.to_string()),
        GameSyncTarget::HighwayHustle => env::var("HIGHWAY_HUSTLE_BACKEND_URL")
            .unwrap_or_else(|_| DEFAULT_HIGHWAY_HUSTLE_BACKEND_URL.to_string()),
        GameSyncTarget::None => String::new(),
    }
}

async fn sync_warzone_entitlement(
    order: &OrderModel,
    listing: &ListingModel,
    item_id: &str,
) -> Result<(), String> {
    let base = game_backend_url(&GameSyncTarget::Warzone);
    let endpoint = format!("{}/", base.trim_end_matches('/'));

    let category = listing.category.trim().to_lowercase();
    let mut payload = json!({
        "walletAddress": order.buyer_wallet,
    });

    if category == "guns" {
        let gun_id: i64 = item_id.parse().map_err(|_| {
            format!(
                "Warzone gun entitlement requires numeric contractItemId, got '{}'",
                item_id
            )
        })?;
        payload["PlayerGuns"] = json!({
            item_id: {
                "id": gun_id,
                "level": 1,
                "ammo": 100000,
                "isNew": true
            }
        });
    } else {
        return Ok(());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed building HTTP client: {}", e))?;

    let response = client
        .post(&endpoint)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Warzone sync request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "Warzone sync failed status={} body={}",
            status, body
        ));
    }

    Ok(())
}
