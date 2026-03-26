# Moments API Reference

Backend routes below are the actual Axum routes for the Moments flow.
If your deployment proxy adds `/api`, prepend `/api` to these routes.

## Auth

Protected endpoints expect:

```http
Authorization: Bearer <jwt>
```

Success responses usually follow:

```json
{
  "ok": true,
  "data": {}
}
```

Error responses usually follow:

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
    "tags": "gameplay,win",
    "search-query": "victory"
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
        "momentId": "4mmj1DLpbU4JO7NlNq5bE",
        "playerWalletAddress": "0xabc...",
        "assetUrl": "https://example.com/moment.png",
        "assetZgHash": "0x123...",
        "assetMetadata": {
          "fileType": "image/png"
        },
        "originalFilename": "moment.png",
        "fileSizeBytes": 245901,
        "title": "Big win",
        "description": "Created from curl",
        "tags": [
          "gameplay",
          "win"
        ],
        "socialMediaLinks": {
          "twitter": "https://x.com/example/status/123456789"
        },
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
        "momentId": "4mmj1DLpbU4JO7NlNq5bE",
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
  "assetUrl": "https://example.com/moment.png",
  "assetMetadata": {
    "fileType": "image/png"
  },
  "socialMediaLinks": {
    "twitter": "https://x.com/example/status/123456789"
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

Case 3: Asset URL not found in storage

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
  "assetUrl": "https://example.com/missing.png"
}
```

Response

```json
{
  "ok": false,
  "message": "Verify failed: File not found in storage"
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
    "assetUrl": "https://example.com/moment.png",
    "assetMetadata": {
      "fileType": "image/png"
    },
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

### Like Moment

#### `POST /moments/{moment_id}/like`

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
    "momentId": "4mmj1DLpbU4JO7NlNq5bE",
    "numLikes": 1,
    "message": "Moment liked successfully"
  }
}
```

Case 2: Already liked

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
  "ok": false,
  "message": "You have already liked this moment"
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
  ],
  "originalFilename": "moment-v2.png",
  "fileSizeBytes": 245901
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
    "originalFilename": "moment-v2.png",
    "fileSizeBytes": 245901,
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

Case 3: Forbidden

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
  "title": "Updated"
}
```

Response

```json
{
  "ok": false,
  "message": "You can only update your own moments"
}
```

Case 4: Not found

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

Case 3: Forbidden

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
  "ok": false,
  "message": "You can only delete your own moments"
}
```

Case 4: Not found

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

## Creators

### My Creator Profile

#### `GET /moments/creators/me`

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
    "walletAddress": "0xabc...",
    "username": "Codex User",
    "rank": 3,
    "totalMoments": 6,
    "totalMomentLikes": 14,
    "totalMomentComments": 9,
    "totalSocialLikes": 20,
    "validatedPostsCount": 2,
    "successfulReferrals": 0,
    "totalScore": 43
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

Case 3: Creator profile not found

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
  "message": "Creator profile not found"
}
```

### Creators Leaderboard

#### `GET /moments/creators/leaderboard`

Case 1: Success

Params

```json
{
  "query": {
    "page": 1,
    "pageSize": 20
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
        "username": "Codex User",
        "totalMoments": 6,
        "totalMomentLikes": 14,
        "totalMomentComments": 9,
        "totalSocialLikes": 20,
        "validatedPostsCount": 2,
        "successfulReferrals": 0,
        "totalScore": 43
      }
    ],
    "totalCount": 12,
    "page": 1,
    "pageSize": 20,
    "totalPages": 1
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

## Social Media

### Submit URL

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

Case 3: Forbidden

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
  "message": "You can only submit posts for your own moments"
}
```

Case 4: Moment not found

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
  "momentId": "missing-id",
  "platform": "Twitter",
  "postId": "tweet-123456789",
  "url": "https://x.com/example/status/123456789"
}
```

Response

```json
{
  "ok": false,
  "message": "Moment not found"
}
```

Case 5: Unauthorized

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

## Notes

- `momentId` is the shareable moment identifier returned by `POST /moments/register`.
- `platform` currently accepts `Twitter`, `Instagram`, `TikTok`, `Facebook`, `Reddit`, `LinkedIn`, `Pinterest`, and `Farcaster`.
- `Farcaster` is accepted by the request enum, but the current Bright Data scraping flow does not support it yet.
- `perPage` defaults to `20` and is capped at `50` on feed endpoints.
- If `assetUrl` is provided on create or update, the backend verifies that the file exists in storage before accepting it.
- Social post validation is asynchronous. `POST /moments/social-media/submit-url` only stores the submission and queues validation work.
