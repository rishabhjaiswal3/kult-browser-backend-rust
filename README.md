# kult-browser-backend-rust

Rust backend service for the Kult Browser platform. Built with [Axum](https://github.com/tokio-rs/axum) and Tokio, it exposes a REST API and runs several background workers for blockchain activity, social-media scraping, data availability (DA), and AI analysis.

## Architecture overview

```
HTTP (Axum)
  ├── /api/health
  ├── /api/content        – game content & section configs
  ├── /api/games          – game catalogue
  ├── /api/leaderboard    – global & per-game leaderboards
  ├── /api/player         – player accounts + SIWE auth
  ├── /api/marketplace    – listings & orders
  ├── /api/moments        – user moments (create, feed, likes, comments)
  ├── /api/moments/social-media – social post scraping jobs
  ├── /api/upload/presign – presigned DO Spaces URLs
  ├── /api/referral       – referral links
  ├── /r/{code}           – referral redirect
  ├── /api/admin          – admin utilities (dev env only)
  └── /docs               – Swagger UI (OpenAPI)

Background workers (Tokio tasks)
  ├── OnchainActivityWorker   – submits activity records to the EVM contract
  ├── DAEventWorker           – uploads event blobs to 0G DA
  ├── ComputeWorker           – AI analysis of moments via 0G Compute
  ├── MigrationWorker         – moment migration pipeline (Valkey queue)
  ├── PostScrapeWorker        – social-media scraping via Bright Data (Valkey queue)
  └── ReferralEvaluationWorker – referral verification & reward logic
```

Storage dependencies: MongoDB (primary store), Valkey/Redis (queues + anti-fraud state), DigitalOcean Spaces (media), 0G Storage (decentralised DA).

## Prerequisites

| Tool | Version |
|------|---------|
| Rust | 1.91.0 (see `rust-toolchain.toml`) |
| MongoDB | 6+ |
| Valkey / Redis | 7+ |

Optional (enable specific workers):

- **Bright Data** account for social-media scraping
- **DigitalOcean Spaces** bucket for media uploads
- **0G Storage CLI** binary for decentralised storage (`0g-storage-client`)
- **0G DA** disperser endpoint for data availability
- **0G Compute** provider for AI moment analysis
- EVM-compatible RPC + deployed contracts for onchain activity recording

## Local development

```bash
# 1. Copy and fill in the environment file
cp .env.example .env   # create this if it doesn't exist — see Environment variables below

# 2. Build and run (debug)
cargo run

# 3. Run dev-only test binaries (requires --features dev-bins)
cargo run --features dev-bins --bin test_bright_data
cargo run --features dev-bins --bin test_post_verification
cargo run --features dev-bins --bin test_full_scrape_pipeline
cargo run --features dev-bins --bin test_0g_upload
cargo run --features dev-bins --bin test_moment_migration_pipeline
cargo run --features dev-bins --bin test_prod_flow
cargo run --features dev-bins --bin test_marketplace_flow
```

The server starts on `http://0.0.0.0:3000` by default. Swagger UI is available at `/docs`.

Admin routes (`/api/admin`) are only mounted when `ENVIRONMENT=dev`.

## Docker

The Dockerfile is a multi-stage build:

1. **zg-builder** (Go) – compiles `0g-storage-client` from source.
2. **rust-builder** – compiles the Rust binary with `--release`.
3. **runtime** (Debian slim) – copies both binaries into a minimal image.

```bash
docker build -t kult-browser-backend-rust .
docker run --env-file .env -p 8080:8080 kult-browser-backend-rust
```

The container exposes port `8080` and defaults `HOST=0.0.0.0`.

## Environment variables

All configuration is loaded at startup via `src/config/`. Variables without a default are **required**.

### Application

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | HTTP listen port |
| `HOST` | `0.0.0.0` | Bind address |
| `APP_NAME` | `kult-browser-backend` | Application name |
| `ENVIRONMENT` / `APP_ENV` | `prod` | Runtime environment; set to `dev` to enable admin routes |
| `CORS_ORIGINS` | `*` | Comma-separated allowed origins |
| `LOG_LEVEL` | `info` | Tracing log level |

### Authentication (JWT + SIWE)

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_SECRET` | `dev-secret-change-me` | JWT signing secret — **change in production** |
| `JWT_EXPIRATION_DAYS` | `7` | Token lifetime in days |
| `SIWE_DOMAIN` | `app.kultgames.io` | Domain shown in the SIWE message |
| `SIWE_URI` | `https://app.kultgames.io` | URI shown in the SIWE message |
| `SIWE_CHAIN_ID` | `1` | EVM chain ID for SIWE |

### MongoDB

| Variable | Default | Description |
|----------|---------|-------------|
| `MONGO_URI` | — | MongoDB connection URI |
| `MONGO_DB_NAME` | — | Database name |
| `MONGO_CONN_RETRIES` | `5` | Connection retry attempts |

Collection names can be overridden via: `GAMES_COLL`, `CONTENT_COLL`, `CAMPAIGNS_COLL`, `CHATBOT_COLL`, `PLAYERS_COLL`, `GLOBAL_LEADERBOARD_COLL`, `GAME_LEADERBOARD_CONFIG_COLL`, `AI_MODELS_COLL`, `MOMENTS_COLL`, `MOMENT_COMMENTS_COLL`, `MOMENT_LIKES_COLL`, `SHARED_POSTS_COLL`, `MARKETPLACE_LISTINGS_COLL`, `MARKETPLACE_ORDERS_COLL`, `ONCHAIN_ACTIVITY_JOBS_COLL`, `PLAYER_NONCES_COLL`.

### Valkey / Redis (queues)

| Variable | Default | Description |
|----------|---------|-------------|
| `VALKEY_URL` | `redis://127.0.0.1:6379` | Connection URL (supports `rediss://` for TLS) |
| `VALKEY_KEY_PREFIX` | `kult_browser_rust` | Prefix applied to all backend-managed keys |

When Valkey is unavailable the server starts without the queue-backed workers and logs a warning.

### DigitalOcean Spaces (media storage)

| Variable | Default | Description |
|----------|---------|-------------|
| `DO_SPACES_KEY` | — | Access Key ID |
| `DO_SPACES_SECRET` | — | Secret Access Key |
| `DO_SPACES_ENDPOINT` | — | Spaces endpoint, e.g. `https://nyc3.digitaloceanspaces.com` |
| `DO_SPACES_REGION` | — | Region, e.g. `nyc3` |
| `MOMENTS_DO_SPACES_BUCKET` | — | Bucket name |
| `MOMENTS_DOWNLOAD_TMP_DIR` | `/tmp/moments` | Temp dir for downloaded files |
| `MOMENTS_DO_SPACES_PRESIGNED_EXPIRATION` | `300` | Presigned URL lifetime in seconds |
| `MOMENTS_UPLOAD_PATH` | `moments` | Upload path prefix inside the bucket |

### Bright Data (social-media scraping)

| Variable | Default | Description |
|----------|---------|-------------|
| `BD_API_KEY` | — | Bearer token for the Bright Data API |
| `BD_BASE_URL` | `https://api.brightdata.com` | Base API URL |
| `BD_POLL_INTERVAL` | `10` | Seconds between snapshot status polls |
| `BD_POLL_TIMEOUT` | `180` | Max seconds to wait for a snapshot |
| `BD_DATASET_TWITTER` | `gd_lwxkxvnf1cynvib9co` | Twitter scraper dataset ID |
| `BD_DATASET_INSTAGRAM` | `gd_lk5ns7kz21pck8jpis` | Instagram scraper dataset ID |
| `BD_DATASET_TIKTOK` | `gd_lu702nij2f790tmv9h` | TikTok scraper dataset ID |
| `BD_DATASET_FACEBOOK` | `gd_lyclm1571iy3mv57zw` | Facebook scraper dataset ID |
| `BD_DATASET_REDDIT` | `gd_lvz8ah06191smkebj4` | Reddit scraper dataset ID |
| `BD_DATASET_LINKEDIN` | `gd_lyy3tktm25m4avu764` | LinkedIn scraper dataset ID |
| `BD_DATASET_PINTEREST` | `gd_lk0sjs4d21kdr7cnlv` | Pinterest scraper dataset ID |

### 0G Storage & DA

| Variable | Default | Description |
|----------|---------|-------------|
| `ZG_PRIVATE_KEY` | — | Private key for signing 0G storage transactions |
| `ZG_BINARY_PATH` | `./0g-storage-client` | Path to the `0g-storage-client` binary |
| `ZG_RPC_URL` | `https://evmrpc.0g.ai/` | 0G EVM RPC endpoint |
| `ZG_INDEXER_URL` | `https://indexer-storage-turbo.0g.ai` | 0G storage indexer |
| `ZG_RPC_TIMEOUT` | `800s` | RPC call timeout |
| `ZG_RPC_RETRY_COUNT` | `5` | RPC retry attempts |
| `ZG_RPC_RETRY_INTERVAL` | `3s` | Delay between RPC retries |
| `ZG_DA_DISPERSER_URL` | — | 0G DA HTTP gateway URL; enables the DA event worker when set |
| `ZG_GATEWAY_URL` | — | URL template for viewing files by hash (use `{hash}` placeholder) |
| `ZG_EXPLORER_TX_URL` | — | URL template for viewing transactions (use `{txHash}` placeholder) |

### 0G Compute (AI analysis)

| Variable | Default | Description |
|----------|---------|-------------|
| `ZG_COMPUTE_PROVIDER_URL` | — | Provider service base URL; enables the compute worker when set |
| `ZG_COMPUTE_API_KEY` | — | Bearer token (`app-sk-<SECRET>`) |
| `ZG_COMPUTE_MODEL` | `gpt-4o-mini` | Model name as reported by the provider |

### Onchain activity

| Variable | Default | Description |
|----------|---------|-------------|
| `ONCHAIN_ENABLED` | `false` | Set to `true`, `1`, or `yes` to enable the onchain worker |
| `ONCHAIN_RPC_URL` | `https://evmrpc.0g.ai/` | EVM RPC endpoint (falls back to `ZG_RPC_URL`) |
| `ONCHAIN_CHAIN_ID` | `16661` | EVM chain ID |
| `ONCHAIN_ACTIVITY_CONTRACT` | — | Address of the deployed activity contract |
| `ONCHAIN_RELAYER_PRIVATE_KEY` | — | Private key used to sign and submit transactions |
| `ONCHAIN_CONFIRMATIONS` | `1` | Blocks to wait for confirmation |
| `ONCHAIN_POLL_INTERVAL_SECS` | `5` | Polling interval in seconds |
| `ONCHAIN_MAX_RETRIES` | `5` | Max submission retries per job |

## Rate limiting

All routes share a token-bucket limiter: sustained ~30 requests/minute per IP, burst up to 60. Stale IP buckets are pruned every 60 seconds.

## Smart contracts

Solidity contracts live in `contracts/`:

- `KultMomentsActivityRecorder.sol` – records moment activity onchain
- `UnifiedGameMarketplace.sol` – marketplace escrow and settlement

Deployment scripts are in `deploy-onchain/`.

## API documentation

Interactive Swagger UI is served at `/docs` when the server is running. The raw OpenAPI JSON is available at `/openapi.json`.

For a static reference see [`docs/app/api_reference.md`](docs/app/api_reference.md).
