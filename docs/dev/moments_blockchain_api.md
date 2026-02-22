# Moments Module - Blockchain Integration Guide

This document outlines the architecture, data models, and API endpoints for the **Moments** module in the `kult-browser-backend` to assist in designing the corresponding Smart Contracts and handling on-chain data verification.

> [!NOTE] 
> The Moments module allows players to capture, store, and share exciting gameplay moments (screenshots, small GIFs) to a public feed, and optionally share them across external social media platforms like Twitter.

## Core Concepts

1. **Moments**: A core asset. It requires an `asset_url` (hosted on DigitalOcean Spaces) and metadata. It is tied to a specific player's [wallet_address](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/src/moments/social_media/repository/post_repository.rs#33-48). 
2. **0G Migration**: When a Moment is successfully created with an asset URL, the backend queues a background job to upload/migrate this asset to the 0G Storage Network to generate a persistent, decentralized `asset_zg_hash` for the file.
3. **Social Sharing**: Players can share their Moments to other platforms (Twitter/Farcaster). We track engagements (likes, score) on these shared posts to fuel the Global Leaderboard.

---

## 1. Data Models

### [Moment](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/src/moments/model/moment_model.rs#11-73) Entity

This is the central data structure stored in the MongoDB [moments](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/src/moments/service/moments_service.rs#212-246) collection.

```json
{
  "_id": "ObjectId",
  "momentId": "string (NanoID, 21 chars)",     // Unique Shareable ID
  "playerWalletAddress": "string",             // Lowercase owner wallet
  "assetUrl": "string?",                       // Link to centralized storage (DO Spaces)
  "originalFilename": "string?", 
  "fileSizeBytes": "number?",
  "assetZgHash": "string?",                    // Populated async by 0G migration worker
  "assetMetadata": "Document?",                // e.g. { "fileType": "image/png" }
  "title": "string (Max 200 chars)",
  "description": "string? (Max 2000 chars)",
  "tags": ["string"],                          // Max 10 tags
  "socialMediaLinks": "Document?",
  "createdAt": "DateTime",
  "updatedAt": "DateTime"
}
```

### [SharedPost](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/src/moments/social_media/model/post_model.rs#14-49) Entity

This tracks a single instance of a player sharing a Moment to an external platform.

```json
{
  "_id": "ObjectId",
  "moment_id": "ObjectId",                     // Reference to the Moment
  "wallet_address": "string",                  // Player identity
  "platform": "string (Twitter|Farcaster|...)",
  "post_id": "string",                         // The tweet/post ID on that platform
  "url": "string",                             // Link to the post
  "num_likes": "number",                       // Updated by background scraper
  "score": "number",                           // Updated by background scraper
  "is_validated": "boolean",
  "validation_status": "string (Pending|Valid|Invalid)",
  "last_validated_at": "DateTime?",
  "createdAt": "DateTime",
  "updatedAt": "DateTime"
}
```

> [!TIP]
> **Blockchain Implication:** The `assetZgHash` is the primary candidate for an on-chain URI or NFT metadata pointer since it represents the immutable data. The `num_likes` and `score` from the [SharedPost](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/src/moments/social_media/model/post_model.rs#14-49) entity will dictate off-chain mechanics that might need to be resolved/verified on-chain for rewards.

---

## 2. Exposed API Endpoints

The API is mounted at `/moments` (or equivalent base router depending on versioning).

### Create Moment
**`POST /register`**
* **Auth:** Required (Signature usually verified in Middleware)
* **Payload:** `title` (req), `asset_url` (req), `description`, `tags`, `asset_metadata`.
* **Logic:** 
  1. Validates string lengths and tag limits.
  2. Generates a NanoID.
  3. Saves the initial Moment to Mongo.
  4. Pushes a message to the Redis `MIGRATION_QUEUE` to asynchronously upload the asset to 0G.

### Get Public Feed
**`GET /`**
* **Auth:** None
* **Params:** `page` (default 1), `per_page` (max 50, default 10), `tags` (optional filter).
* **Returns:** A paginated list of all public Moments.

### Get Player Moments
**`GET /my`**
* **Auth:** Required
* **Params:** `page`, `per_page`
* **Returns:** A paginated list of Moments owned by the calling wallet.

### Get Single Moment
**`GET /:moment_id`**
* **Auth:** None
* **Returns:** The full Moment object including the `asset_zg_hash` if the migration task has finished.

### Update Moment
**`PATCH /:moment_id`**
* **Auth:** Required
* **Logic:** Fails immediately if the caller's wallet does not match the `player_wallet_address` on the Moment. Updates text fields, metadata, or the `asset_url`. *Note: If `asset_url` is updated, the file must already exist in DO Spaces, otherwise the update is rejected.*

### Delete Moment
**`DELETE /:moment_id`**
* **Auth:** Required
* **Logic:** Validates ownership via wallet address before permanently deleting the document from the database.

### Get Sharing Leaderboard `[IN-PROGRESS]`
**`GET /leaderboard`** (Exact path TBD)
* **Auth:** None
* **Params:** `page`, `per_page`
* **Returns:** A sorted, paginated list of players ranked descending by their total accrued `score` across all validated [SharedPost](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/src/moments/social_media/model/post_model.rs#14-49) records.
* **Blockchain Implication:** This is the endpoint that the Smart Contract Oracles (or the players themselves submitting proofs) will rely on to distribute token rewards based on a player's social media sharing and engagement. It aggregates the data populated by the Scraper worker below.

---

## 3. Asynchronous Workers

### 0G Migration Worker
A background job consumes the `MIGRATION_QUEUE` Redis stream whenever a new Moment is created. 
It pulls the file from the centralized `assetUrl`, uploads it via the 0G Storage Client, returns the distributed hash, and updates the `assetZgHash` on the Moment document.

### Social Media Scraper
*(In Development)* A background job that takes [SharedPost](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/src/moments/social_media/model/post_model.rs#14-49) records, calls external APIs (e.g. BrightData scrapers), and validates the engagement metrics (Likes) to update the `num_likes` and `score` fields. This is critical for preventing fraudulent engagement farming before we award leaderboard points.
