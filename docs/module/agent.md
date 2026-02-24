# Agent Module — `src/agent/`


## Overview

The Agent module creates and manages **AI Agents as Web3 entities**. Every user gets an autonomous AI agent paired to their account upon registration — the agent has its own EVM wallet and can (eventually) transact on-chain independently.

---

## Directory Structure

```
src/agent/
├── mod.rs                          # Module exports
├── wallet.rs                       # EVM wallet generation (alloy)
├── model/
│   ├── mod.rs
│   └── agent_model.rs              # AgentModel MongoDB struct
└── repository/
    ├── mod.rs
    └── agent_repository.rs         # DB operations + auto-indexes
```

---

## Components

### 1. Wallet Generator — `wallet.rs`

Uses the **`alloy`** crate (`alloy-signer-local`) — the industry-standard Rust EVM library by Paradigm (successor to `ethers-rs`).

| Method | Description |
|---|---|
| `AgentWallet::generate()` | Generates a secure random SECP256K1 keypair (private key + EVM address) |
| `AgentWallet::from_private_key(hex)` | Reconstructs wallet from a stored hex private key |

### 2. Data Model — `model/agent_model.rs`

MongoDB collection: configurable via `AI_MODELS_COLL` env var (default: `ai_models`)

| Field | BSON Key | Type | Default | Description |
|---|---|---|---|---|
| `id` | `_id` | ObjectId | auto | MongoDB document ID |
| `owner_wallet` | `ownerWallet` | String | — | Human user's wallet address |
| `agent_wallet` | `agentWallet` | String | — | Agent's generated EVM address |
| `private_key` | `privateKey` | String | — | Agent's raw private key |
| `token_id` | `tokenId` | Option | None | NFT Token ID (if minted on AIRegistry) |
| `config_cid` | `configCid` | Option | None | 0G Storage hash for agent brain config |
| `core_permission_level` | `corePermissionLevel` | u8 | 1 | Autonomy: 1=Observe, 2=Recommend, 3=Confirm, 4=Autonomous |
| `elo_rating` | `eloRating` | u32 | 1200 | Skill rating (provisional) |
| `reputation_score` | `reputationScore` | u32 | 100 | Trust score |
| `suspension_level` | `suspensionLevel` | u8 | 0 | Anti-cheat: 0=Active → 4=Banned |
| `nonce` | `nonce` | u64 | 0 | Blockchain transaction counter |
| `created_at` | `createdAt` | DateTime | now | Creation timestamp |
| `updated_at` | `updatedAt` | DateTime | now | Last update timestamp |

### 3. Repository — `repository/agent_repository.rs`

| Method | Description |
|---|---|
| `new(db)` | Initializes collection from config + creates unique indexes |
| `create_agent_for_new_user(owner_wallet)` | Generates wallet, builds model, inserts into DB |
| `get_agent_by_owner(owner_wallet)` | Find agent by human user's wallet |
| `get_agent_by_wallet(agent_wallet)` | Find agent by the agent's own wallet |

**Auto-created indexes:**
- `ownerWallet` — unique (one agent per user)
- `agentWallet` — unique (globally unique agent wallets)

---

## Integration Point

Wired into the **Player Registration flow** in `src/player/`:

```
POST /api/player/login
  └─ PlayerService::login()
       └─ if is_new_player:
            └─ AgentRepository::create_agent_for_new_user(wallet)
                 └─ AgentWallet::generate()  →  AgentModel::new()  →  insert
```

The agent creation is **silent** — the user doesn't see or interact with it. If agent generation fails, the user login still succeeds (error is logged but not blocking).

---

## Known Gaps / Pending Work

| Item | Status | Notes |
|---|---|---|
| Chain ID & network per wallet | ❌ Pending | Currently assumes single EVM chain |
| Multi-chain wallet derivation | ❌ Pending | Need per-chain deposit addresses (ETH, SOL, 0G) |
| KMS integration for key storage | ❌ Pending | Private key is currently plain text in MongoDB — must migrate to GCP KMS / Fireblocks |
| Internal ledger (off-chain balance) | ❌ Pending | `available`, `locked`, `pending` balance tracking |
| Deposit/withdrawal tables | ❌ Pending | `deposits`, `withdrawals`, `ledger_entries` collections |
| Blockchain listeners | ❌ Pending | Per-chain listeners to detect incoming deposits |
| Fund sweeping | ❌ Pending | Auto-sweep from deposit addresses to pool wallet |
| Agent autonomous transactions | ❌ Pending | Entry fees, rewards, staking, contract interactions |
| Smart contracts per chain | ❌ Pending | AIRegistry, TournamentRegistry, SettlementRouter |
