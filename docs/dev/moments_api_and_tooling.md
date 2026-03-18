# Moments API And Tooling Guide

This document reflects the current implementation under `src/moments` and the supporting upload, queue, and worker code used by the Moments feature in `kult-browser-backend-rust`.

## What this module does

The Moments module supports four related flows:

1. A player uploads an image or GIF to DigitalOcean Spaces through a presigned URL.
2. The client registers a moment record in MongoDB.
3. A background worker optionally migrates the uploaded asset to 0G storage and writes the resulting `assetZgHash` back onto the moment.
4. A player can submit a social post URL for later scraping and validation through Bright Data.

## Important implementation notes

These are the main things another dev is likely to trip over:

- The create route is `POST /api/moments/register`, not `POST /api/moments`.
- The public feed is the root route mounted under `/api/moments`.
- `momentId` returned by the moments API is a NanoID share ID. The social sharing endpoint currently expects a MongoDB `ObjectId` string in `momentId`, not that NanoID.
- Social post submission does not currently verify that the referenced moment exists or belongs to the authenticated wallet.
- `Platform::Farcaster` is accepted by the enum, but the Bright Data scraper explicitly does not support Farcaster. Those jobs will fail and eventually go to the scrape dead-letter queue.
- On create, a migration job is queued only when `assetUrl` is non-empty and `assetMetadata.fileType` exists.
- On create, `assetUrl` is not checked against Spaces. On update, it is checked.
- On update, changing `assetUrl` checks that the file exists in Spaces, but it does not queue a fresh 0G migration job.
- If Valkey is unavailable, moment creation and social post submission still save to MongoDB, but the async workers are effectively disabled because no jobs are queued.

## Router mount points

The server mounts these relevant routers:

- `/api/moments`
- `/api/moments/social-media`
- `/api/upload`

The mounting happens in `src/server.rs`.

## Auth and response shape

Protected endpoints use the `AuthPlayer` extractor and expect:

```http
Authorization: Bearer <jwt>
```

Most successful responses use:

```json
{
  "ok": true,
  "data": {}
}
```

Most failures use:

```json
{
  "ok": false,
  "message": "error text"
}
```

Auth failures also use the same error shape.

## End-to-end client flow

### 1. Request a presigned upload URL

Endpoint:

```http
POST /api/upload/presign
```

Auth: required

Request body:

```json
{
  "filename": "clutch-win.png",
  "contentType": "image/png"
}
```

Success payload:

```json
{
  "ok": true,
  "data": {
    "upload_url": "https://...",
    "public_url": "https://<bucket>.<region>.digitaloceanspaces.com/moments/clutch-win.png",
    "required_headers": {
      "x-amz-acl": "public-read"
    }
  }
}
```

Use `upload_url` plus the returned `required_headers` to upload directly to Spaces, then use `public_url` as `assetUrl` when creating or updating the moment.

### 2. Register the moment

Endpoint:

```http
POST /api/moments/register
```

Auth: required

Request body:

```json
{
  "assetUrl": "https://<bucket>.<endpoint>/moments/clutch-win.png",
  "assetMetadata": {
    "fileType": "image/png",
    "width": 1920,
    "height": 1080
  },
  "title": "Clutch ranked finish",
  "description": "Final hit with 3 HP left",
  "tags": ["ranked", "boss", "gif"],
  "socialMediaLinks": {
    "twitter": "https://x.com/example/status/123"
  }
}
```

Validation rules:

- `title` is required and cannot exceed 200 chars.
- `description` cannot exceed 2000 chars.
- `tags` cannot exceed 10 entries.
- `assetUrl` is optional on create.
- `assetMetadata.fileType` must exist if you want the 0G migration job to be queued.

Success payload:

```json
{
  "ok": true,
  "data": {
    "momentId": "V1StGXR8_Z5jdHi6B-myT",
    "message": "Moment created successfully"
  }
}
```

What happens internally:

- A `MomentModel` is inserted into the Mongo `moments` collection.
- `playerWalletAddress` is stored lowercased.
- If Valkey is available and the request has both `assetUrl` and `assetMetadata.fileType`, a `MigrationJob` is pushed to `moments:migration`.

### 3. Read moments

#### Public feed

Endpoint:

```http
GET /api/moments?page=1&perPage=20&tags=ranked,boss
```

Auth: not required

Behavior:

- `page` defaults to `1`.
- `perPage` defaults to `20`.
- `perPage` is capped at `50`.
- `tags` is a comma-separated list. A moment matches when it has at least one of the supplied tags.

Success payload:

```json
{
  "ok": true,
  "data": {
    "moments": [
      {
        "momentId": "V1StGXR8_Z5jdHi6B-myT",
        "playerWalletAddress": "0xabc...",
        "assetUrl": "https://...",
        "assetZgHash": "0x...",
        "assetMetadata": {
          "fileType": "image/png"
        },
        "title": "Clutch ranked finish",
        "description": "Final hit with 3 HP left",
        "tags": ["ranked", "boss"],
        "socialMediaLinks": {
          "twitter": "https://x.com/example/status/123"
        },
        "createdAt": "2026-03-17T08:10:00Z",
        "updatedAt": "2026-03-17T08:12:00Z"
      }
    ],
    "total": 1,
    "page": 1,
    "perPage": 20
  }
}
```

#### Authenticated player's moments

Endpoint:

```http
GET /api/moments/my?page=1&perPage=20
```

Auth: required

Behavior:

- Uses the wallet from the JWT.
- Same pagination rules as the public feed.

#### Single moment

Endpoint:

```http
GET /api/moments/:moment_id
```

Auth: not required

Behavior:

- `:moment_id` is the NanoID share ID returned by `POST /api/moments/register`.
- Returns `404` if the moment is not found.

### 4. Update a moment

Endpoint:

```http
PATCH /api/moments/:moment_id
```

Auth: required

Request body: partial update, all fields optional

```json
{
  "assetUrl": "https://<bucket>.<endpoint>/moments/clutch-win-v2.png",
  "originalFilename": "clutch-win-v2.png",
  "fileSizeBytes": 245901,
  "assetMetadata": {
    "fileType": "image/png"
  },
  "title": "Clutch ranked finish v2",
  "description": "Higher-res upload",
  "tags": ["ranked", "boss"],
  "socialMediaLinks": {
    "twitter": "https://x.com/example/status/123"
  }
}
```

Behavior:

- Only the owner can update.
- If `assetUrl` is provided, it must be non-empty and must already exist in DigitalOcean Spaces.
- `title`, `description`, and `tags` use the same max-length rules as create.
- If the request is empty, the service returns the existing moment unchanged.

Current caveat:

- Updating `assetUrl` does not enqueue a new migration job, so `assetZgHash` will not automatically refresh.

### 5. Delete a moment

Endpoint:

```http
DELETE /api/moments/:moment_id
```

Auth: required

Behavior:

- Only the owner can delete.
- Success response:

```json
{
  "ok": true,
  "data": {
    "message": "Moment deleted successfully"
  }
}
```

### 6. Submit a social media post for validation

Endpoint:

```http
POST /api/moments/social-media/submit-url
```

Auth: required

Request body:

```json
{
  "momentId": "67d6dfdcfa6a9e22efc04f2b",
  "platform": "Twitter",
  "postId": "1901234567890",
  "url": "https://x.com/example/status/1901234567890"
}
```

Supported platform strings:

- `Twitter`
- `Instagram`
- `TikTok`
- `Facebook`
- `Reddit`
- `LinkedIn`
- `Pinterest`
- `Farcaster`

Success payload:

```json
{
  "ok": true,
  "data": {
    "postId": "67d6e02cfa6a9e22efc04f31",
    "message": "Post submitted successfully. Validation will be processed shortly."
  }
}
```

Behavior:

- Duplicate protection is global on `(platform, post_id)`.
- The inserted document goes into the `shared_posts` collection by default, or the collection named by `SHARED_POSTS_COLL`.
- The record starts with:
  - `num_likes = 0`
  - `score = 0`
  - `is_validated = false`
  - `validation_status = Pending`
- If Valkey is available, a `ScrapeJob` is pushed to `posts:scrape`.

Current caveats:

- The `momentId` field here is currently parsed as a Mongo `ObjectId`, not the shareable NanoID used by the rest of the moments API.
- The service does not currently verify that the referenced moment exists.
- The service does not currently verify that the authenticated player owns the referenced moment.

## Data model summary

### Moment document

Mongo collection: `moments`

Important fields:

- `_id`: Mongo `ObjectId`
- `momentId`: public NanoID used by the CRUD API
- `playerWalletAddress`: lowercased wallet address
- `assetUrl`: public DigitalOcean Spaces URL
- `assetZgHash`: 0G root hash written asynchronously by the migration worker
- `assetMetadata`: arbitrary JSON converted to BSON
- `title`, `description`, `tags`
- `socialMediaLinks`: arbitrary JSON converted to BSON
- `originalFilename`, `fileSizeBytes`
- `createdAt`, `updatedAt`

### SharedPost document

Mongo collection: `shared_posts` by default

Important fields:

- `_id`: Mongo `ObjectId`
- `moment_id`: Mongo `ObjectId` reference as currently implemented
- `wallet_address`
- `platform`
- `post_id`
- `url`
- `num_likes`
- `score`
- `is_validated`
- `validation_status`
- `validation_reason`
- `last_validated_at`
- `created_at`, `updated_at`

Field semantics to note:

- `is_validated` means the scraper/validator has processed the post.
- Whether the post passed validation is represented by `validation_status` (`Pending`, `Valid`, `Invalid`).

## Background workers and external tools

### A. 0G migration pipeline

Purpose:

- Move a moment asset from DigitalOcean Spaces into 0G storage.
- Persist the returned 0G root hash into `assetZgHash`.

Queue names:

- Main queue: `moments:migration`
- Processing queue: `moments:migration_processing`
- Dead-letter queue: `moments:dead`

Job shape:

```json
{
  "assetUrl": "https://...",
  "assetId": "V1StGXR8_Z5jdHi6B-myT",
  "assetType": "image/png",
  "attempt": 1
}
```

Worker flow:

1. Pop with the reliable queue pattern (`BRPOPLPUSH`).
2. Download the file from Spaces.
3. Run the `0g-storage-client` CLI through `external/0g/storage/upload.rs`.
4. Parse the returned root hash and optional tx hash.
5. Update the Mongo moment document by `momentId`.
6. Ack the queue job.

Retry behavior:

- Hard-coded max retries: `3`
- Failed jobs after max retries go to `moments:dead`

Required config for this path:

- `VALKEY_URL`
- `DO_SPACES_KEY`
- `DO_SPACES_SECRET`
- `DO_SPACES_ENDPOINT`
- `DO_SPACES_REGION`
- `MOMENTS_DO_SPACES_BUCKET`
- `MOMENTS_DOWNLOAD_TMP_DIR`
- `ZG_BINARY_PATH`
- `ZG_RPC_URL`
- `ZG_PRIVATE_KEY`
- `ZG_INDEXER_URL`
- `ZG_RPC_TIMEOUT`
- `ZG_RPC_RETRY_COUNT`
- `ZG_RPC_RETRY_INTERVAL`

### B. Social post scrape and validation pipeline

Purpose:

- Scrape the submitted post later.
- Decide whether it is a valid Kult-related post.
- Update likes and score on the `SharedPost`.

Queue names:

- Main queue: `posts:scrape`
- Processing queue: `posts:scrape_processing`
- Dead-letter queue: `posts:scrape:dead`

Job shape:

```json
{
  "post_db_id": "67d6e02cfa6a9e22efc04f31",
  "platform": "Twitter",
  "url": "https://x.com/example/status/1901234567890",
  "created_at": "2026-03-17T08:20:00Z",
  "attempt": 1
}
```

Worker flow:

1. Pop a `ScrapeJob` from `posts:scrape`.
2. Enforce a minimum post age gate using `SCRAPE_MIN_AGE_HOURS` before scraping.
3. Call `BrightDataPostScraper::scrape_post`.
4. Normalize raw platform-specific data into a common `ScrapedPostData` shape.
5. Run `PostValidator` against hashtags, URL fields, and text content.
6. Set `num_likes`.
7. Set `score = likes`.
8. Update `is_validated`, `validation_status`, `validation_reason`, and timestamps.
9. Ack the queue job.

Validation logic:

- Valid if hashtags include `kultgames`, `kult.games`, or `kult`
- Valid if an extracted URL contains `kult.games`
- Valid if text contains `kult.games` or `@kultgames`
- Otherwise marked `Invalid`

Retry behavior:

- Configured by `SCRAPE_MAX_RETRIES`
- Failed jobs after max retries go to `posts:scrape:dead`

Required config for this path:

- `VALKEY_URL`
- `BD_API_KEY`
- `BD_BASE_URL`
- `BD_TRIGGER_PATH`
- `BD_PROGRESS_PATH`
- `BD_SNAPSHOT_PATH`
- `BD_POLL_INTERVAL`
- `BD_POLL_TIMEOUT`
- `BD_DATASET_TWITTER`
- `BD_DATASET_INSTAGRAM`
- `BD_DATASET_TIKTOK`
- `BD_DATASET_FACEBOOK`
- `BD_DATASET_REDDIT`
- `BD_DATASET_LINKEDIN`
- `BD_DATASET_PINTEREST`
- `SCRAPE_MIN_AGE_HOURS`
- `SCRAPE_MAX_RETRIES`
- `SCRAPE_POLL_TIMEOUT_SECS`
- `SCRAPE_REQUEUE_SLEEP_SECS`

## Internal touchpoints for backend devs

If another Rust dev needs the main entry points, these are the ones that matter:

- `moments::routes(db, migration_queue)` builds the CRUD router.
- `social_media::route::routes(scrape_queue).await` builds the social submission router.
- `MomentsService` owns CRUD logic and optional migration queueing.
- `PostService` owns social post submission and optional scrape queueing.
- `MigrationWorker` processes 0G migration jobs.
- `PostScrapeWorker` processes delayed scrape jobs.
- `ValkyQueue` implements the reliable queue behavior used by both workers.

## Practical recommendations before another team integrates this

- Treat this document as the current implementation contract, not the intended product contract.
- If the consumer only knows the public `momentId` NanoID, they cannot reliably call the social submit endpoint without an additional lookup or API change.
- Do not expose Farcaster in a client until the scraper support exists or the backend rejects it at submit time.
- If `assetUrl` is ever updated after create, add a follow-up backend change to requeue migration; the current code does not do that.
- If social submission is security-sensitive, add moment existence and ownership checks before relying on it.
