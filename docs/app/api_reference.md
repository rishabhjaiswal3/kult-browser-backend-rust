# App API Reference

Backend routes below are the actual Axum routes.
If your deployment proxy adds `/api`, prepend `/api` to these routes.
The referral redirect route remains `/r/{code}`.

## Health

### Liveness

#### `GET /health`

Case 1: Success

Params

```text
None
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "ts": "2026-03-20T06:28:03.649866+00:00"
}
```

## Content

### Section Content

#### `GET /content`

Case 1: Section found

Params

```json
{
  "query": {
    "page": "home",
    "section": "top_picks",
    "page_num": 1,
    "page_size": 10
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "content": [
      {
        "_id": "object_id_hex",
        "identification": "zerogpool",
        "name": {
          "default": "en",
          "en": "Zero G Pool"
        }
      }
    ],
    "total_content_count": 1,
    "page": 1,
    "page_size": 10
  }
}
```

Case 2: Section not found

Params

```json
{
  "query": {
    "page": "home",
    "section": "hero"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "Section 'hero' not found on page 'home'"
}
```

Case 3: Invalid query

Params

```json
{
  "query": {
    "page_num": "abc"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "Failed to deserialize query string"
}
```

## Games

### All Games

#### `GET /games/all`

Case 1: Normal list with optional search

Params

```json
{
  "query": {
    "search": "pool",
    "page": 1,
    "page_size": 10
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "games": [
      {
        "identification": "zerogpool",
        "name": {
          "default": "en",
          "en": "Zero G Pool"
        },
        "thumbnail": {
          "horizontal": {
            "default": "en",
            "en": {
              "url": "https://example.com/image.png"
            }
          },
          "vertical": {
            "default": "en",
            "en": {
              "url": "https://example.com/image-mobile.png"
            }
          }
        },
        "category": "sports"
      }
    ],
    "totalCount": 9,
    "page": 1,
    "pageSize": 10,
    "totalPages": 1
  }
}
```

Case 2: Backend failure

Params

```json
{
  "query": {
    "search": "pool"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

### All Categories

#### `GET /games/all-categories`

Case 1: Success

Params

```text
None
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "categories": [
      "sports",
      "arcade"
    ]
  }
}
```

Case 2: Backend failure

Params

```text
None
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

### Game Detail

#### `GET /games/{identification}`

Case 1: Game found

Params

```json
{
  "path": {
    "identification": "zerogpool"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "game": {
      "identification": "zerogpool",
      "name": {
        "default": "en",
        "en": "Zero G Pool"
      },
      "url": "https://example.com/play",
      "thumbnail": {
        "horizontal": {
          "default": "en",
          "en": {
            "url": "https://example.com/image.png"
          }
        },
        "vertical": {
          "default": "en",
          "en": {
            "url": "https://example.com/image-mobile.png"
          }
        }
      },
      "about": {
        "default": "en",
        "en": "Game description"
      }
    }
  }
}
```

Case 2: Not found

Params

```json
{
  "path": {
    "identification": "missing-game"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "Game 'missing-game' not found"
}
```

## Leaderboard

### Global Leaderboard

#### `GET /leaderboard/global`

Case 1: Success

Params

```json
{
  "query": {
    "page": 1,
    "page_size": 50
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "entries": [
      {
        "rank": 1,
        "walletAddress": "0xabc...",
        "score": 1200.5,
        "level": 8
      }
    ],
    "totalCount": 1491,
    "page": 1,
    "pageSize": 50,
    "totalPages": 30
  }
}
```

Case 2: Invalid query

Params

```json
{
  "query": {
    "page": "abc"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

### Game Leaderboard

#### `GET /leaderboard/game/{identification}`

Case 1: Success

Params

```json
{
  "path": {
    "identification": "zerogpool"
  },
  "query": {
    "page": 1,
    "page_size": 50
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "entries": [
      {
        "rank": 1,
        "player": "0xabc...",
        "score": 450.0,
        "level": 3,
        "metadata": {
          "streak": 12
        }
      }
    ],
    "totalCount": 300,
    "page": 1,
    "pageSize": 50,
    "totalPages": 6
  }
}
```

Case 2: Config or game not found

Params

```json
{
  "path": {
    "identification": "missing-game"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

### Refresh

#### `POST /leaderboard/refresh`

Case 1: Success

Params

```text
None
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "refreshed": 1491,
    "message": "Refreshed 1491 entries"
  }
}
```

Case 2: Backend failure

Params

```text
None
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

## Player

### Login

#### `POST /player/login`

Case 1: Login or create player

Params

```text
None
```

Request Body

```json
{
  "walletAddress": "0x1234567890abcdef1234567890abcdef12345678",
  "name": "Codex User",
  "metadata": {
    "source": "curl"
  },
  "referralCode": "abcd1234"
}
```

Response

```json
{
  "ok": true,
  "data": {
    "token": "jwt string",
    "player": {
      "id": "object_id_hex",
      "walletAddress": "0x1234567890abcdef1234567890abcdef12345678",
      "name": "Codex User"
    }
  }
}
```

Case 2: Bad request

Params

```text
None
```

Request Body

```json
{
  "walletAddress": ""
}
```

Response

```json
{
  "ok": false,
  "message": "walletAddress is required"
}
```

### Profile

#### `GET /player/profile`

Case 1: Success

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "cached": false,
    "profile": {
      "walletAddress": "0x1234567890abcdef1234567890abcdef12345678",
      "username": "Codex User",
      "rank": 10,
      "totalScore": 2500.5,
      "level": 7,
      "totalGamesPlayed": 3,
      "completedQuests": 0,
      "gameScoresList": [
        {
          "identification": "zerogpool",
          "score": 500.0,
          "weight": 1.0,
          "weightedScore": 500.0,
          "rank": 7
        }
      ]
    }
  }
}
```

Case 2: Unauthorized

Params

```text
Missing Authorization header
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

Case 3: Player not found

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "Player not found"
}
```

### Update Name

#### `PATCH /player/name`

Case 1: Success

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```json
{
  "name": "Codex User Updated"
}
```

Response

```json
{
  "ok": true,
  "data": {
    "name": "Codex User Updated"
  }
}
```

Case 2: Invalid body or unauthorized

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```json
{
  "name": ""
}
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

## Moments

### Public Feed

#### `GET /moments`

Case 1: Success

Params

```json
{
  "query": {
    "page": 1,
    "perPage": 20,
    "tags": "gameplay,win"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "moments": [
      {
        "momentId": "abc123",
        "playerWalletAddress": "0xabc...",
        "title": "Big win",
        "description": "Test moment",
        "tags": [
          "gameplay",
          "win"
        ],
        "createdAt": "2026-03-20T00:00:00Z",
        "updatedAt": "2026-03-20T00:00:00Z"
      }
    ],
    "total": 1,
    "page": 1,
    "perPage": 20
  }
}
```

Case 2: Backend failure

Params

```json
{
  "query": {
    "page": 1
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

### My Moments

#### `GET /moments/my`

Case 1: Success

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  },
  "query": {
    "page": 1,
    "perPage": 20
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "moments": [
      {
        "momentId": "abc123",
        "playerWalletAddress": "0xabc...",
        "title": "My moment",
        "tags": [
          "test"
        ],
        "createdAt": "2026-03-20T00:00:00Z",
        "updatedAt": "2026-03-20T00:00:00Z"
      }
    ],
    "total": 1,
    "page": 1,
    "perPage": 20
  }
}
```

Case 2: Unauthorized

Params

```text
Missing Authorization header
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

### Register

#### `POST /moments/register`

Case 1: Success

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```json
{
  "title": "My Moment",
  "description": "Created from curl",
  "tags": [
    "test",
    "moment"
  ],
  "assetUrl": "https://example.com/image.png",
  "assetMetadata": {
    "fileType": "image/png"
  },
  "socialMediaLinks": {
    "twitter": "https://x.com/example"
  }
}
```

Response

```json
{
  "ok": true,
  "data": {
    "momentId": "4mmj1DLpbU4JO7NlNq5bE",
    "message": "Moment created successfully"
  }
}
```

Case 2: Invalid body

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```json
{
  "title": ""
}
```

Response

```json
{
  "ok": false,
  "message": "title is required"
}
```

### Get Moment

#### `GET /moments/{moment_id}`

Case 1: Success

Params

```json
{
  "path": {
    "moment_id": "4mmj1DLpbU4JO7NlNq5bE"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "momentId": "4mmj1DLpbU4JO7NlNq5bE",
    "playerWalletAddress": "0xabc...",
    "title": "My Moment",
    "description": "Created from curl",
    "tags": [
      "test",
      "moment"
    ],
    "createdAt": "2026-03-20T00:00:00Z",
    "updatedAt": "2026-03-20T00:00:00Z"
  }
}
```

Case 2: Not found

Params

```json
{
  "path": {
    "moment_id": "missing-id"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "Moment not found"
}
```

### Update Moment

#### `PATCH /moments/{moment_id}`

Case 1: Success

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  },
  "path": {
    "moment_id": "4mmj1DLpbU4JO7NlNq5bE"
  }
}
```

Request Body

```json
{
  "title": "My Moment Updated",
  "description": "Updated from curl",
  "tags": [
    "updated",
    "moment"
  ]
}
```

Response

```json
{
  "ok": true,
  "data": {
    "momentId": "4mmj1DLpbU4JO7NlNq5bE",
    "title": "My Moment Updated",
    "description": "Updated from curl",
    "tags": [
      "updated",
      "moment"
    ],
    "createdAt": "2026-03-20T00:00:00Z",
    "updatedAt": "2026-03-20T00:10:00Z"
  }
}
```

Case 2: Unauthorized or invalid body

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  },
  "path": {
    "moment_id": "4mmj1DLpbU4JO7NlNq5bE"
  }
}
```

Request Body

```json
{
  "title": ""
}
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

Case 3: Not found

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  },
  "path": {
    "moment_id": "missing-id"
  }
}
```

Request Body

```json
{
  "title": "Updated"
}
```

Response

```json
{
  "ok": false,
  "message": "Moment not found"
}
```

### Delete Moment

#### `DELETE /moments/{moment_id}`

Case 1: Success

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  },
  "path": {
    "moment_id": "4mmj1DLpbU4JO7NlNq5bE"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": true,
  "data": {
    "message": "Moment deleted successfully"
  }
}
```

Case 2: Unauthorized

Params

```text
Missing Authorization header
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

Case 3: Not found

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  },
  "path": {
    "moment_id": "missing-id"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "Moment not found"
}
```

### Social Media Submit URL

#### `POST /moments/social-media/submit-url`

Case 1: Success

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```json
{
  "momentId": "4mmj1DLpbU4JO7NlNq5bE",
  "platform": "Twitter",
  "postId": "tweet-123456789",
  "url": "https://x.com/example/status/123456789"
}
```

Response

```json
{
  "ok": true,
  "data": {
    "postId": "object_id_hex",
    "message": "Post submitted successfully. Validation will be processed shortly."
  }
}
```

Case 2: Duplicate post

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```json
{
  "momentId": "4mmj1DLpbU4JO7NlNq5bE",
  "platform": "Twitter",
  "postId": "tweet-123456789",
  "url": "https://x.com/example/status/123456789"
}
```

Response

```json
{
  "ok": false,
  "message": "This post has already been submitted"
}
```

Case 3: Unauthorized

Params

```text
Missing Authorization header
```

Request Body

```json
{
  "momentId": "4mmj1DLpbU4JO7NlNq5bE",
  "platform": "Twitter",
  "postId": "tweet-123456789",
  "url": "https://x.com/example/status/123456789"
}
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

## Upload

### Presign

#### `POST /upload/presign`

Case 1: Success

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```json
{
  "filename": "moment.png",
  "content_type": "image/png"
}
```

Response

```json
{
  "ok": true,
  "data": {
    "upload_url": "https://...",
    "public_url": "https://...",
    "required_headers": {
      "content-type": "image/png"
    }
  }
}
```

Case 2: Unauthorized

Params

```text
Missing Authorization header
```

Request Body

```json
{
  "filename": "moment.png",
  "content_type": "image/png"
}
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

Case 3: Presign failure

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```json
{
  "filename": "moment.png",
  "content_type": "image/png"
}
```

Response

```json
{
  "upload_url": "",
  "public_url": "",
  "required_headers": {}
}
```

## Referral

### My Link

#### `GET /referral/me`

Case 1: Success

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "code": "wjd6egri",
  "link": "https://klt.gm/r/wjd6egri"
}
```

Case 2: Unauthorized

Params

```text
Missing Authorization header
```

Request Body

```text
None
```

Response

```json
{
  "ok": false,
  "message": "string"
}
```

Case 3: Referral link generation failed

Params

```json
{
  "headers": {
    "Authorization": "Bearer <jwt>"
  }
}
```

Request Body

```text
None
```

Response

```json
{
  "error": "Could not generate referral link"
}
```

### Redirect

#### `GET /r/{code}`

Case 1: Redirect success

Params

```json
{
  "path": {
    "code": "wjd6egri"
  }
}
```

Request Body

```text
None
```

Response

```http
HTTP/1.1 302 Found
Location: https://kult.games/join?ref=wjd6egri
```
