// src/mongo/indexes.rs
// Ensures MongoDB indexes exist at startup to prevent full collection scans.

use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use mongodb::Database;
use mongodb::IndexModel;

use crate::config::CONFIG;

/// Create all required indexes. Called once at startup.
/// Uses `create_index` which is idempotent — safe to call repeatedly.
pub async fn ensure_indexes(db: &Database) {
    let results = tokio::join!(
        ensure_player_indexes(db),
        ensure_game_indexes(db),
        ensure_leaderboard_indexes(db),
        ensure_moments_indexes(db),
        ensure_social_media_indexes(db),
        ensure_content_indexes(db),
        ensure_agent_indexes(db),
        ensure_referral_indexes(db),
        ensure_onchain_indexes(db),
    );

    // Log any failures but don't crash — the app can still work without indexes (just slower)
    let labels = [
        "player",
        "game",
        "leaderboard",
        "moments",
        "social_media",
        "content",
        "agent",
        "referral",
        "onchain",
    ];
    for (i, result) in [
        results.0, results.1, results.2, results.3, results.4, results.5, results.6, results.7,
        results.8,
    ]
    .into_iter()
    .enumerate()
    {
        if let Err(e) = result {
            tracing::error!(module = labels[i], error = %e, "Failed to create indexes");
        }
    }

    tracing::info!("MongoDB index initialization complete");
}

async fn ensure_onchain_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    let coll =
        db.collection::<mongodb::bson::Document>(&CONFIG.db.onchain_activity_jobs_collection);

    coll.create_index(
        IndexModel::builder()
            .keys(doc! { "activityId": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
    )
    .await?;

    coll.create_index(
        IndexModel::builder()
            .keys(doc! { "status": 1, "createdAt": 1 })
            .build(),
    )
    .await?;

    coll.create_index(
        IndexModel::builder()
            .keys(doc! { "userWallet": 1, "createdAt": -1 })
            .build(),
    )
    .await?;

    Ok(())
}

async fn ensure_player_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    let coll = db.collection::<mongodb::bson::Document>(&CONFIG.db.players_collection);

    coll.create_index(
        IndexModel::builder()
            .keys(doc! { "walletAddress": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
    )
    .await?;

    coll.create_index(
        IndexModel::builder()
            .keys(doc! { "referralCode": 1 })
            .options(IndexOptions::builder().sparse(true).build())
            .build(),
    )
    .await?;

    Ok(())
}

async fn ensure_game_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    let coll = db.collection::<mongodb::bson::Document>(&CONFIG.db.games_collection);

    coll.create_index(
        IndexModel::builder()
            .keys(doc! { "identification": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
    )
    .await?;

    Ok(())
}

async fn ensure_leaderboard_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    // Global leaderboard
    let global = db.collection::<mongodb::bson::Document>(&CONFIG.db.global_leaderboard_collection);

    global
        .create_index(
            IndexModel::builder()
                .keys(doc! { "walletAddress": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    global
        .create_index(IndexModel::builder().keys(doc! { "score": -1 }).build())
        .await?;

    // Game leaderboard configs
    let config =
        db.collection::<mongodb::bson::Document>(&CONFIG.db.game_leaderboard_config_collection);

    config
        .create_index(
            IndexModel::builder()
                .keys(doc! { "identification": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    Ok(())
}

async fn ensure_moments_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    let moments = db.collection::<mongodb::bson::Document>(&CONFIG.db.moments_collection);

    moments
        .create_index(
            IndexModel::builder()
                .keys(doc! { "momentId": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    moments
        .create_index(
            IndexModel::builder()
                .keys(doc! { "playerWalletAddress": 1, "createdAt": -1 })
                .build(),
        )
        .await?;

    moments
        .create_index(
            IndexModel::builder()
                .keys(doc! { "relatedGames": 1, "createdAt": -1 })
                .build(),
        )
        .await?;

    // Comments
    let comments = db.collection::<mongodb::bson::Document>(&CONFIG.db.moment_comments_collection);

    comments
        .create_index(
            IndexModel::builder()
                .keys(doc! { "momentId": 1, "parentCommentId": 1, "createdAt": 1 })
                .build(),
        )
        .await?;

    // Likes (unique per moment+wallet)
    let likes = db.collection::<mongodb::bson::Document>(&CONFIG.db.moment_likes_collection);

    likes
        .create_index(
            IndexModel::builder()
                .keys(doc! { "momentId": 1, "authorWalletAddress": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    Ok(())
}

async fn ensure_social_media_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    let posts = db.collection::<mongodb::bson::Document>(&CONFIG.db.shared_posts_collection);

    posts
        .create_index(
            IndexModel::builder()
                .keys(doc! { "wallet_address": 1, "created_at": -1 })
                .build(),
        )
        .await?;

    posts
        .create_index(
            IndexModel::builder()
                .keys(doc! { "platform": 1, "post_id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    Ok(())
}

async fn ensure_content_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    let coll = db.collection::<mongodb::bson::Document>(&CONFIG.db.content_collection);

    coll.create_index(
        IndexModel::builder()
            .keys(doc! { "page": 1, "section": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
    )
    .await?;

    Ok(())
}

async fn ensure_agent_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    let coll = db.collection::<mongodb::bson::Document>(&CONFIG.db.ai_models_collection);

    coll.create_index(
        IndexModel::builder()
            .keys(doc! { "ownerWallet": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
    )
    .await?;

    coll.create_index(
        IndexModel::builder()
            .keys(doc! { "agentWallet": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
    )
    .await?;

    Ok(())
}

async fn ensure_referral_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    let connections = db.collection::<mongodb::bson::Document>("store_referrals");

    connections
        .create_index(
            IndexModel::builder()
                .keys(doc! { "referred_player_id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    let scores = db.collection::<mongodb::bson::Document>("store_referral_scores");

    scores
        .create_index(
            IndexModel::builder()
                .keys(doc! { "walletAddress": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    Ok(())
}
