# Marketplace Module

## Public Endpoints

### GET `/marketplace`

List active listings (paginated, filterable).

**Query Params**

| Param | Type | Default | Description |
|---|---|---|---|
| `gameIdentification` | string | — | Filter by game slug |
| `assetType` | string | — | Filter by asset type |
| `page` | u32 | 1 | Page number |
| `perPage` | u32 | 20 | Items per page (max 100) |

**Response** `200`

```json
{
  "ok": true,
  "data": {
    "listings": [
      {
        "id": "ObjectId",
        "name": "Golden Sword",
        "description": "Limited edition weapon",
        "assetType": "weapon",
        "gameIdentification": "highway-hustle",
        "thumbnailUrl": "https://...",
        "price": 1.5,
        "supply": 100,
        "remaining": 97,
        "status": "active",
        "attributes": { "rarity": "legendary", "damage": 150 },
        "createdAt": "2026-04-15T00:00:00Z",
        "updatedAt": "2026-04-15T00:00:00Z"
      }
    ],
    "total": 42,
    "page": 1,
    "perPage": 20
  }
}
```

---

### GET `/marketplace/:id`

Get single listing by ID.

**Path Params**

| Param | Type | Description |
|---|---|---|
| `id` | string | Listing ObjectId |

**Response** `200`

```json
{
  "ok": true,
  "data": {
    "id": "ObjectId",
    "name": "Golden Sword",
    "description": "Limited edition weapon",
    "assetType": "weapon",
    "gameIdentification": "highway-hustle",
    "thumbnailUrl": "https://...",
    "price": 1.5,
    "supply": 100,
    "remaining": 97,
    "status": "active",
    "attributes": { "rarity": "legendary" },
    "createdAt": "2026-04-15T00:00:00Z",
    "updatedAt": "2026-04-15T00:00:00Z"
  }
}
```

**Errors:** `400` invalid ID format, `404` not found

---

## Order Endpoints (Auth Required)

All order endpoints require `Authorization: Bearer <token>` header.

### POST `/marketplace/orders`

Purchase a listing.

**Request Body**

```json
{
  "listingId": "ObjectId",
  "quantity": 1,
  "txHash": "0x..."
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `listingId` | string | yes | — | Listing to purchase |
| `quantity` | u32 | no | 1 | Quantity to buy |
| `txHash` | string | no | — | On-chain tx hash |

**Response** `200`

```json
{
  "ok": true,
  "data": {
    "id": "ObjectId",
    "listingId": "ObjectId",
    "playerId": "player_id",
    "gameIdentification": "highway-hustle",
    "pricePaid": 1.5,
    "quantity": 1,
    "status": "completed",
    "txHash": "0x...",
    "createdAt": "2026-04-15T00:00:00Z"
  }
}
```

**Errors:** `400` invalid listingId / zero quantity, `401` unauthorized, `404` listing not found, `409` sold out

---

### GET `/marketplace/orders`

Get authenticated player's order history.

**Query Params**

| Param | Type | Default | Description |
|---|---|---|---|
| `page` | u32 | 1 | Page number |
| `perPage` | u32 | 20 | Items per page (max 100) |

**Response** `200`

```json
{
  "ok": true,
  "data": {
    "orders": [
      {
        "id": "ObjectId",
        "listingId": "ObjectId",
        "playerId": "player_id",
        "gameIdentification": "highway-hustle",
        "pricePaid": 1.5,
        "quantity": 1,
        "status": "completed",
        "txHash": "0x...",
        "createdAt": "2026-04-15T00:00:00Z"
      }
    ],
    "total": 5,
    "page": 1,
    "perPage": 20
  }
}
```

---

### GET `/marketplace/orders/:id`

Get single order (must belong to authenticated player).

**Path Params**

| Param | Type | Description |
|---|---|---|
| `id` | string | Order ObjectId |

**Response** `200` — same shape as single order above

**Errors:** `400` invalid ID, `401` unauthorized, `403` not your order, `404` not found

---

## Admin Endpoints

Requires `ENVIRONMENT=dev` on server. No auth.

### POST `/admin/marketplace`

Create a new listing.

**Request Body**

```json
{
  "name": "Golden Sword",
  "description": "Limited edition weapon",
  "assetType": "weapon",
  "gameIdentification": "highway-hustle",
  "thumbnailUrl": "https://...",
  "price": 1.5,
  "supply": 100,
  "attributes": { "rarity": "legendary", "damage": 150 }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Listing name (non-empty) |
| `description` | string | no | Description |
| `assetType` | string | yes | Asset category |
| `gameIdentification` | string | yes | Game slug (non-empty) |
| `thumbnailUrl` | string | no | Thumbnail image URL |
| `price` | f64 | yes | Price (>= 0) |
| `supply` | u64 | no | Total supply (omit for unlimited) |
| `attributes` | object | no | Flexible key-value metadata |

**Response** `200` — full `ListingResponse` (same as GET single listing)

**Errors:** `400` empty name / negative price / empty gameIdentification

---

### PUT `/admin/marketplace/:id`

Update an existing listing. All fields optional, at least one required.

**Path Params**

| Param | Type | Description |
|---|---|---|
| `id` | string | Listing ObjectId |

**Request Body**

```json
{
  "name": "Updated Sword",
  "price": 2.0,
  "description": "Now even more legendary"
}
```

| Field | Type | Description |
|---|---|---|
| `name` | string | New name (non-empty if provided) |
| `description` | string | New description |
| `assetType` | string | New asset type |
| `thumbnailUrl` | string | New thumbnail URL |
| `price` | f64 | New price (>= 0) |
| `supply` | u64 | New supply (also resets remaining) |
| `attributes` | object | New attributes |

**Response** `200` — full `ListingResponse`

**Errors:** `400` no fields / empty name / negative price, `404` not found

---

### DELETE `/admin/marketplace/:id`

Soft-delete: sets listing status to `delisted`.

**Path Params**

| Param | Type | Description |
|---|---|---|
| `id` | string | Listing ObjectId |

**Response** `200`

```json
{
  "ok": true,
  "data": {
    "id": "ObjectId",
    "message": "Listing delisted successfully"
  }
}
```

**Errors:** `400` invalid ID, `404` not found

---

### GET `/admin/marketplace/orders`

Get all orders across all players (paginated).

**Query Params**

| Param | Type | Default | Description |
|---|---|---|---|
| `page` | u32 | 1 | Page number |
| `perPage` | u32 | 20 | Items per page (max 100) |

**Response** `200` — same shape as `OrderListResponse` (all players' orders)
