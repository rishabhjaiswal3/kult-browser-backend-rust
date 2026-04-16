# Marketplace API

## Listing Object

```json
{
  "id": "6660...",
  "name": "AWP",
  "shortDescription": "Powerful sniper rifle",
  "longDescription": "The AWP is a high-powered sniper rifle...",
  "assetUrl": "https://cdn.example.com/gun-awp.png",
  "price": 4.0,
  "category": "Guns",
  "currency": "SOMI",
  "gameIdentification": "warzonewarriors",
  "status": "active"
}
```

Optional fields (`shortDescription`, `longDescription`, `assetUrl`) are omitted when null.

## Order Object

```json
{
  "id": "6660...",
  "listingId": "6660...",
  "playerId": "6660...",
  "gameIdentification": "warzonewarriors",
  "pricePaid": 4.0,
  "quantity": 1,
  "status": "completed",
  "txHash": "0x..."
}
```

`txHash` omitted when null.

---

## Public

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/marketplace` | List active listings |
| GET | `/marketplace/:id` | Get single listing |

**GET `/marketplace`** query params: `gameIdentification`, `category`, `page` (default 1), `perPage` (default 20, max 100)

Response: `{ ok, data: { listings: [...], total, page, perPage } }`

---

## Orders (Bearer token required)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/marketplace/orders` | Purchase a listing |
| GET | `/marketplace/orders` | My order history |
| GET | `/marketplace/orders/:id` | Get single order |

**POST `/marketplace/orders`** body:

| Field | Type | Required | Default |
|-------|------|----------|---------|
| `listingId` | string | yes | — |
| `quantity` | u32 | no | 1 |
| `txHash` | string | no | — |

**GET `/marketplace/orders`** query params: `page`, `perPage`

Response: `{ ok, data: { orders: [...], total, page, perPage } }`

Errors: `400` bad input, `401` no token, `403` not your order, `404` not found

---

## Admin (requires `ENVIRONMENT=dev`, no auth)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/admin/marketplace` | Create listing |
| PUT | `/admin/marketplace/:id` | Update listing |
| DELETE | `/admin/marketplace/:id` | Delist (soft-delete) |
| GET | `/admin/marketplace/orders` | All orders (paginated) |

**POST `/admin/marketplace`** body:

| Field | Type | Required |
|-------|------|----------|
| `name` | string | yes |
| `shortDescription` | string | no |
| `longDescription` | string | no |
| `assetUrl` | string | no |
| `price` | f64 | yes |
| `category` | string | yes |
| `currency` | string | yes |
| `gameIdentification` | string | yes |

**PUT `/admin/marketplace/:id`** — same fields as create, all optional, at least one required.

**DELETE `/admin/marketplace/:id`** — returns `{ ok, data: { id, message } }`

**GET `/admin/marketplace/orders`** — same query/response shape as player orders.
