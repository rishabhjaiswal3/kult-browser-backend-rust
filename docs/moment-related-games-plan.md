# Moment Related Games Implementation Plan

## Goal

Replace the `External URL` and `Platform` sections in the Kult Moment create dialog with a `Related Games` multi-select dropdown.

Important contract decision:

- Store only game `identification` values on moments.
- Do not store game names on moments.
- Game names are only used in the create dropdown to help the user choose the right game.
- Once a moment is created, related games should display like tags using the identification, for example `#guesstheai`.

## Where Identifications Come From

The selected game identifications come from the existing games catalog API.

Frontend API wrapper:

```text
browser-deployed/kult-moment/src/lib/games-api.ts
```

Function:

```text
listAllGames
```

Backend endpoint:

```text
GET /api/games/all
```

Frontend path:

```text
GET /games/all
```

Query params:

```text
page
page_size
search
```

Each returned game item already has:

```text
identification
name
thumbnail
isDownloadable
category
slogan
rating
```

The dropdown will use:

- `game.identification` as the real selected value.
- `resolveLocalizedText(game.name)` only for the create dropdown option label.

Example:

```text
Display label: Guess The AI
Stored value: guesstheai
```

Backend validation, if enabled, should validate submitted IDs against the games collection using the `identification` field.

## Data Model Decision

Add this field to moments:

```text
relatedGames: string[]
```

Each string is a game `identification`.

Example stored document:

```text
relatedGames: ["guesstheai", "highway-hustle"]
```

Why only identification:

- It is the stable key already used by games APIs.
- It avoids duplicated game names in moment documents.
- It prevents stale display names if game names change.
- The moment UI stays simple and displays identifications directly like tags.

## Current State

### Frontend

Create dialog:

```text
browser-deployed/kult-moment/src/components/CreateMomentModal.tsx
```

Current create submit sends:

```text
title
description
tags
assetUrl
assetMetadata
socialMediaLinks
```

Moment API wrapper:

```text
browser-deployed/kult-moment/src/lib/moments-api.ts
```

Create endpoint:

```text
POST /moments/register
```

### Backend

Moment model:

```text
browser-deployed/kult-browser-backend-rust/src/moments/model/moment_model.rs
```

Current model includes:

```text
momentId
playerWalletAddress
assetUrl
assetZgHash
numLikes
numComments
assetMetadata
title
description
tags
socialMediaLinks
createdAt
updatedAt
```

Moment DTOs:

```text
src/moments/dto/create_moment.rs
src/moments/dto/update_moment.rs
src/moments/dto/moment_response.rs
```

Moment service:

```text
src/moments/service/moments_service.rs
```

Moment repository:

```text
src/moments/repository/moments_repository.rs
```

## Backend API Contract

### Create Moment

Endpoint:

```text
POST /api/moments/register
```

Frontend path:

```text
POST /moments/register
```

Auth:

```text
Authorization: Bearer <token>
```

New request body:

```text
{
  title: string,
  description?: string,
  tags: string[],
  assetUrl: string,
  assetMetadata?: {
    fileType?: string
  },
  relatedGames: string[]
}
```

Example:

```text
{
  title: "My clutch round",
  description: "Won the match with one health left.",
  tags: ["clutch", "arena"],
  assetUrl: "https://cdn.example.com/moments/abc.gif",
  assetMetadata: {
    fileType: "image/gif"
  },
  relatedGames: ["guesstheai", "highway-hustle"]
}
```

Removed from frontend create request:

```text
socialMediaLinks
```

Create response stays unchanged:

```text
{
  momentId: string,
  message: string
}
```

Actual backend envelope remains:

```text
{
  ok: true,
  data: {
    momentId: string,
    message: string
  }
}
```

### Get Moment

Endpoint:

```text
GET /api/moments/{momentId}
```

Moment response should include:

```text
relatedGames: string[]
```

Example response data:

```text
{
  momentId: "abc123",
  playerWalletAddress: "0x123...",
  assetUrl: "https://cdn.example.com/moments/abc.gif",
  assetZgHash: null,
  numLikes: 0,
  numComments: 0,
  assetMetadata: {
    fileType: "image/gif"
  },
  title: "My clutch round",
  description: "Won the match with one health left.",
  tags: ["clutch", "arena"],
  relatedGames: ["guesstheai", "highway-hustle"],
  socialMediaLinks: null,
  createdAt: "2026-04-14T09:10:00Z",
  updatedAt: "2026-04-14T09:10:00Z"
}
```

### List Moments

Endpoints:

```text
GET /api/moments
GET /api/moments/my
```

Every returned moment item should include:

```text
relatedGames: string[]
```

Existing list shape remains:

```text
{
  moments: MomentResponse[],
  total: number,
  page: number,
  perPage: number
}
```

### Update Moment

Endpoint:

```text
PATCH /api/moments/{momentId}
```

Add optional request field:

```text
relatedGames?: string[]
```

Behavior:

- If `relatedGames` is absent, keep existing related games unchanged.
- If `relatedGames` is an empty array, clear related games.
- If `relatedGames` has values, normalize and store the identifications.

## Backend Implementation Plan

### 1. Update MomentModel

File:

```text
src/moments/model/moment_model.rs
```

Add:

```text
related_games: Vec<String>
```

Serde:

- Rename as `relatedGames`.
- Default to empty array for old documents.

Theory:

- Old moments in MongoDB do not have this field.
- `#[serde(default)]` allows old documents to deserialize as `relatedGames: []`.
- No blocking migration is required.

### 2. Update CreateMomentRequest

File:

```text
src/moments/dto/create_moment.rs
```

Add:

```text
related_games: Vec<String>
```

Serde:

- Existing `rename_all = "camelCase"` exposes it as `relatedGames`.
- Use default empty array.

Keep `social_media_links` temporarily for compatibility, but stop sending it from the frontend create flow.

### 3. Update UpdateMomentRequest

File:

```text
src/moments/dto/update_moment.rs
```

Add:

```text
related_games: Option<Vec<String>>
```

This allows PATCH to distinguish:

- field absent: no change
- field present as empty array: clear values

### 4. Update MomentResponse

File:

```text
src/moments/dto/moment_response.rs
```

Add:

```text
related_games: Vec<String>
```

Response field:

```text
relatedGames
```

Do not skip serializing when empty. Return `[]` for consistency.

### 5. Normalize And Validate Related Game Identifications

File:

```text
src/moments/service/moments_service.rs
```

Add helper logic similar to tag normalization:

- Trim each identification.
- Remove empty values.
- Deduplicate values.
- Cap maximum count.

Recommended max:

```text
5 related games per moment
```

Recommended backend validation:

- Validate every submitted identification against the games collection.
- Source of truth is the games collection field named `identification`.
- Use `GameRepository::exists` or equivalent lookup by `identification`.

Bad request cases:

- More than the maximum allowed related games.
- Any submitted identification is empty after trimming.
- Any submitted identification does not exist in the games collection, if validation is enabled.

### 6. Store Related Games On Create

File:

```text
src/moments/service/moments_service.rs
```

In `create_moment`, populate the `MomentModel` field:

```text
related_games
```

from normalized request identifications.

### 7. Store Related Games On Update

File:

```text
src/moments/service/moments_service.rs
```

In `update_moment`, handle:

```text
request.related_games
```

Mongo update key:

```text
relatedGames
```

### 8. Map Related Games In Responses

File:

```text
src/moments/service/moments_service.rs
```

In `to_response`, map:

```text
moment.related_games -> response.related_games
```

### 9. Search Support

File:

```text
src/moments/repository/moments_repository.rs
```

Existing search matches:

```text
title
description
tags
```

Update search to also match:

```text
relatedGames
```

Since only identifications are stored, backend moment search can match `guesstheai` or `highway-hustle`, not game display names like `Guess The AI`.

If name-based search is required later, it should be implemented by resolving game names to identifications first, then filtering by those identifications.

### 10. Optional Game Filter

Not required for the first UI request, but useful later.

Potential query:

```text
GET /api/moments?games=guesstheai,highway-hustle
```

Filter:

```text
relatedGames in selected game identifications
```

### 11. Add Mongo Index

File:

```text
src/mongo/indexes.rs
```

Add index:

```text
relatedGames: 1
createdAt: -1
```

Purpose:

- Makes future game-based filtering efficient.
- Helps any future game-detail page show related moments quickly.

## Frontend Implementation Plan

### 1. Remove External URL And Platform UI

File:

```text
browser-deployed/kult-moment/src/components/CreateMomentModal.tsx
```

Remove:

- `platforms` constant.
- `externalUrl` state.
- `selectedPlatform` state.
- External URL validation.
- Platform validation.
- External URL field section.
- Platform button grid.
- `socialMediaLinks` from create payload.

### 2. Add Related Games State

Create selected games state as only identifications:

```text
selectedGameIds: string[]
```

Reset it in `resetForm`.

The create dropdown can use the loaded games list to show readable names during selection, but selected values are still stored as identifications only.

### 3. Fetch Games

Use existing:

```text
listAllGames
```

Recommended query:

```text
GET /games/all?page=1&page_size=100
```

Frontend receives:

```text
{
  games,
  totalCount,
  page,
  pageSize,
  totalPages
}
```

Use:

- `game.identification` as the option value and submitted value.
- `resolveLocalizedText(game.name)` as the create dropdown option label.

### 4. Multi-Select Dropdown UI

Add field:

```text
RELATED GAMES
```

Behavior:

- Click opens dropdown.
- Dropdown lists games.
- Each row shows readable game name and `identification`.
- Clicking a game toggles its `identification` in `selectedGameIds`.
- Selected games show as removable chips.
- Selected chips should show identifications like tags.

Suggested dropdown option label:

```text
Guess The AI · guesstheai
```

Suggested selected chip label:

```text
#guesstheai
```

Stored and submitted value:

```text
guesstheai
```

### 5. Create Submit Payload

Update `CreateMomentInput` in:

```text
browser-deployed/kult-moment/src/lib/moments-api.ts
```

Add:

```text
relatedGames: string[]
```

Remove from create input:

```text
socialMediaLinks
```

Update request body:

```text
{
  title,
  description,
  tags,
  assetUrl,
  assetMetadata,
  relatedGames
}
```

### 6. Update Moment Type

File:

```text
browser-deployed/kult-moment/src/lib/moments-api.ts
```

Add:

```text
relatedGames: string[]
```

Keep:

```text
socialMediaLinks?: Record<string, string>
```

for backward compatibility with old moments and the social validation panel.

### 7. Moment Display

Recommended:

- Show related game chips on `MomentFeedCard`.
- Show related game chips on `MomentDetailPage`.

Display rule:

- Display related games like tags.
- Prefix each identification with `#`.
- Do not fetch the games catalog just to display moments.
- Do not resolve game names on moment cards or detail pages.

Example:

```text
#guesstheai #highway-hustle
```

## Request And Response Examples

### Games Dropdown Request

```text
GET /api/games/all?page=1&page_size=100
```

Frontend path:

```text
GET /games/all?page=1&page_size=100
```

Response envelope:

```text
{
  ok: true,
  data: {
    games: [
      {
        identification: "guesstheai",
        name: {
          default: "en",
          en: "Guess The AI"
        },
        thumbnail: {},
        isDownloadable: false,
        category: "ai",
        slogan: {
          default: "en",
          en: "Spot the AI image"
        },
        rating: 4.8
      }
    ],
    totalCount: 1,
    page: 1,
    pageSize: 100,
    totalPages: 1
  }
}
```

The frontend uses:

```text
game.identification -> submitted value
game.name -> dropdown option label only
```

### Create Moment Request

```text
POST /api/moments/register
Authorization: Bearer <token>
Content-Type: application/json
```

Body:

```text
{
  title: "My clutch round",
  description: "Won the match with one health left.",
  tags: ["clutch", "arena"],
  assetUrl: "https://cdn.example.com/moments/abc.gif",
  assetMetadata: {
    fileType: "image/gif"
  },
  relatedGames: ["guesstheai", "highway-hustle"]
}
```

Response envelope:

```text
{
  ok: true,
  data: {
    momentId: "abc123",
    message: "Moment created successfully"
  }
}
```

### Get Moment Response

```text
{
  ok: true,
  data: {
    momentId: "abc123",
    playerWalletAddress: "0x123...",
    assetUrl: "https://cdn.example.com/moments/abc.gif",
    assetZgHash: null,
    numLikes: 0,
    numComments: 0,
    assetMetadata: {
      fileType: "image/gif"
    },
    title: "My clutch round",
    description: "Won the match with one health left.",
    tags: ["clutch", "arena"],
    relatedGames: ["guesstheai", "highway-hustle"],
    createdAt: "2026-04-14T09:10:00Z",
    updatedAt: "2026-04-14T09:10:00Z"
  }
}
```

## Compatibility Notes

Keep `socialMediaLinks` in backend model and response for now.

Reasons:

- Existing moments may already have this field.
- `MomentSocialValidationPanel` currently reads `moment.socialMediaLinks`.
- Social validation still has a separate submit URL API:
  - `POST /api/moments/social-media/submit-url`

The create dialog should stop collecting social data. Users can continue submitting public post URLs on the detail page if that flow is still needed.

## Verification Checklist

Backend:

- Create moment with no related games succeeds.
- Create moment with one related game identification succeeds.
- Create moment with multiple related game identifications succeeds.
- Duplicate game identifications store only one entry.
- Empty game identification is rejected or removed during normalization.
- Invalid game identification is rejected if backend game validation is enabled.
- Old moments without `relatedGames` deserialize as `relatedGames: []`.
- Feed response includes `relatedGames`.
- Detail response includes `relatedGames`.
- Update moment can set related games.
- Update moment can clear related games.
- Search matches related game identifications if search support is added.

Frontend:

- External URL field is gone.
- Platform section is gone.
- Related Games dropdown appears.
- Games load from `GET /games/all`.
- Each option shows name and identification.
- Multiple games can be selected.
- Selected games can be removed.
- Selected related game chips display like tags, for example `#guesstheai`.
- Moment cards/details display related games as `#identification` chips without resolving names.
- Create payload sends `relatedGames` as `string[]`.
- Create payload no longer sends `socialMediaLinks`.
- Build passes.

## Recommended Implementation Order

1. Add `related_games: Vec<String>` to the backend moment model.
2. Add `related_games` to create/update/response DTOs.
3. Normalize and store related game identifications in `MomentsService`.
4. Optionally validate identifications against the games collection.
5. Return related game identifications from `to_response`.
6. Add search support for `relatedGames`.
7. Add Mongo index for `relatedGames`.
8. Update frontend moment types.
9. Update frontend create request payload.
10. Replace external URL/platform UI with related games multi-select.
11. Render related game chips on cards/detail as `#identification`, like tags.
12. Run Rust tests/build.
13. Run frontend build.
