# Social Media Post Verification — Kult Browser


## 1. Supported Platforms

| # | Platform | Scraper Provider |
|---|----------|-----------------|
| 1 | Twitter / X | Bright Data |
| 2 | Instagram | Bright Data |
| 3 | TikTok | Bright Data |
| 4 | Facebook | Bright Data |
| 5 | Reddit | Bright Data |
| 6 | LinkedIn | Bright Data |
| 7 | Pinterest | Bright Data |

## 2. Unsupported Platforms

| Platform | Why Unsupported | Workaround |
|----------|----------------|------------|
| Farcaster | Decentralized, no scraper available | Referral link: `https://kult.games/m/{id}?ref={wallet}` — track clicks |
| Snapchat | Ephemeral stories, no public pages | Same referral link |
| WhatsApp | E2E encrypted, no public posts | Same referral link |
| Telegram | No scraper available | Same referral link |
| Discord | Server-gated, not public | Same referral link |

---

## 3. Full BrightData Responses (Live Scrapes)

Each section below shows the **complete raw JSON** returned by BrightData for a real public post.

---

### 3.1 Twitter / X

**Source file:** [twitter.json](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/data/twitter.json)

```json
{
  "id": "2020518068800160166",
  "user_posted": "Ronnie_Ree",
  "name": "Outpost Games",
  "description": "My indie game had 20,000 wishlists.\nVery Positive reviews.\nA huge streamer played it live.\nIt died after 15 days.\n\nI made a video on what exactly I think went wrong.\n\nhttps://t.co/UzoW7HSQPX\n\n#indiegames #indiedev #gamedev",
  "date_posted": "2026-02-08T15:20:21.000Z",
  "photos": null,
  "url": "https://x.com/Ronnie_Ree/status/2020518068800160166",
  "quoted_post": { "photos": null, "videos": null },
  "tagged_users": null,
  "replies": 1,
  "reposts": 1,
  "likes": 4,
  "views": 266,
  "external_url": "https://youtu.be/V0TKrj0j9rg?si=2yyOu7gmDQyyxstm",
  "hashtags": ["indiegames", "indiedev", "gamedev"],
  "followers": 1985,
  "biography": "Discord: https://t.co/f90QYtjgK4 Youtube: https://t.co/GoxgElW0wu",
  "posts_count": 3807,
  "profile_image_link": "https://pbs.twimg.com/profile_images/.../dbx3bm6A_normal.jpg",
  "following": 323,
  "is_verified": true,
  "quotes": 0,
  "bookmarks": 1,
  "parent_post_details": {
    "post_id": "2020518068800160166",
    "profile_id": "3349479671",
    "profile_name": "Outpost Games",
    "date_posted": "2026-02-08T15:20:21.000Z"
  },
  "external_image_urls": null,
  "videos": null,
  "external_video_urls": null,
  "verification_type": "blue",
  "user_id": "3349479671",
  "context_added": null,
  "timestamp": "2026-02-25T17:44:10.251Z",
  "input": { "url": "https://x.com/Ronnie_Ree/status/2020518068800160166" }
}
```

---

### 3.2 Instagram

**Source file:** [instagram_exturl.json](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/data/instagram_exturl.json)

```json
{
  "url": "https://www.instagram.com/reel/CoZBDp2hu0H/",
  "user_posted": "deathmakerhf",
  "description": "Step up your gaming experience with the Cosmic Byte CB-GK-33 Astra! This hot-swappable mechanical keyboard features per key RGB lighting and comes with both wired and Bluetooth connectivity options.\n\nGet ready to unlock new levels of gaming performance!\n\n@thecosmicbyte\n\nTo find out more about this amazing product, check the links below:\nhttps://amzn.to/3JE9O0e\nhttp://bit.ly/3HQjLpT\n\n#CosmicByte #Astra #Vlt #mechanicalkeyboard #rgblighting #wiredandbluetooth #Deathmaker",
  "hashtags": ["#CosmicByte", "#Astra", "#Vlt", "#mechanicalkeyboard", "#rgblighting", "#wiredandbluetooth", "#Deathmaker"],
  "likes": 1377,
  "num_comments": 7,
  "date_posted": "2023-02-08T06:12:17.000Z",
  "photos": ["https://scontent-iad3-2.cdninstagram.com/..."],
  "alt_text": null,
  "error": null,
  "video_view_count": "7886",
  "user_posted_id": "4132985526",
  "followers": 17704,
  "tagged_users": [
    { "full_name": "Cosmic Byte", "id": "9668199440", "is_verified": true, "username": "thecosmicbyte" }
  ],
  "shortcode": "CoZBDp2hu0H",
  "video_play_count": 23901,
  "latest_comments": [
    { "comments": "Good products. Using a Hyperion mouse with alturas keyboard..", "user_commenting": "aftab786157", "likes": 2 },
    { "comments": "Jod🔥🔥", "user_commenting": "ig_official_rajput", "likes": 0 }
  ],
  "is_verified": false,
  "content_type": "Reel",
  "posts_count": 283,
  "is_paid_partnership": false,
  "audio": { "audio_asset_id": "531934389083962", "original_audio_title": "Original audio" },
  "timestamp": "2026-02-24T19:06:01.681Z",
  "input": { "url": "https://www.instagram.com/reel/CoZBDp2hu0H/" }
}
```

---

### 3.3 TikTok

**Source file:** [tiktok.json](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/data/tiktok.json)

```json
{
  "url": "https://www.tiktok.com/@extrahighlandbros/video/7609101937220308255",
  "description": "Check Our Our Latest YouTube Video To See Why We Posted This... #dance #dancetok #funny #highlandbros #fyp",
  "profile_username": "extrahighlandbros",
  "hashtags": ["dance", "dancetok", "funny", "highlandbros", "fyp"],
  "digg_count": 24400,
  "collect_count": 473,
  "comment_count": 58,
  "share_count": 204,
  "play_count": 282800,
  "create_time": "2026-02-21T00:01:56.000Z",
  "video_url": "https://v16-webapp-prime.us.tiktok.com/video/...",
  "error": null,
  "video_duration": 5,
  "profile_url": "https://www.tiktok.com/@extrahighlandbros",
  "region": "US",
  "profile_id": "7361928224698074154",
  "profile_followers": 264500,
  "profile_biography": "Extra @Highland Bros Content\n📥hbros@viralnationtalent.com📥\n⬇️All Our Links⬇️",
  "profile_avatar": "https://p16-common-sign.tiktokcdn-us.com/...",
  "is_verified": false,
  "post_type": "video",
  "post_id": "7609101937220308255",
  "music": { "authorname": "BABE CAVE", "title": "original sound", "original": true },
  "timestamp": "2026-02-24T18:55:01.236Z",
  "input": { "url": "https://www.tiktok.com/@extrahighlandbros/video/7609101937220308255" }
}
```

---

### 3.4 Facebook

**Source file:** [facebook.json](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/data/facebook.json)

```json
{
  "url": "https://www.facebook.com/kotaku/posts/1126531136015163/",
  "post_id": "1126531136015163",
  "user_url": "https://www.facebook.com/kotaku",
  "user_username_raw": "Kotaku",
  "content": "Playing Gears of War On My PS5 Is Sooooo Weird",
  "date_posted": "2025-08-26T16:27:11.000Z",
  "hashtags": [],
  "num_comments": 40,
  "num_shares": 6,
  "num_likes_type": [
    { "type": "Like", "num": 43 },
    { "type": "Haha", "num": 16 },
    { "type": "Care", "num": 1 },
    { "type": "Wow", "num": 1 }
  ],
  "page_followers": 979000,
  "page_is_verified": true,
  "page_logo": "https://scontent.flhr14-1.fna.fbcdn.net/...",
  "page_url": "https://www.facebook.com/kotaku",
  "attachments": [
    { "id": "1126531129348497", "type": "genericattachmentmedia", "url": "https://external.fruh4-4.fna.fbcdn.net/..." }
  ],
  "post_external_link": "https://kotaku.com/gears-of-war-remastered-reloaded-running-ps5-xbox-weird-2000620092",
  "post_external_title": "Playing Gears of War On My PS5 Is So Weird",
  "is_sponsored": false,
  "likes": 43,
  "post_type": "Post shared",
  "timestamp": "2026-02-25T17:45:34.209Z",
  "input": { "url": "https://www.facebook.com/kotaku/posts/1126531136015163/" }
}
```

---

### 3.5 Reddit

**Source file:** [reddit.json](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/data/reddit.json) (trimmed — full file includes 20+ comments and 25 related posts)

```json
{
  "post_id": "t3_1rdppqz",
  "url": "https://www.reddit.com/r/gaming/comments/1rdppqz/1_million_in_debt_devs_on_handheld_tony_hawks_pro/",
  "user_posted": "OhMyOhWhyOh",
  "title": "$1 million in debt, devs on handheld Tony Hawk's Pro Skater saved the company by pitching \"fake\" screenshots that forced them to turn the GBA into a 3D gaming machine: \"Nobody could believe it\"",
  "description": null,
  "num_comments": 330,
  "date_posted": "2026-02-24T19:18:17.329Z",
  "community_name": "gaming",
  "num_upvotes": 7469,
  "photos": null,
  "videos": null,
  "tag": null,
  "embedded_links": [
    "https://www.gamesradar.com/games/sports/usd1-million-in-debt-devs-on-handheld-tony-hawks-pro-skater-saved-the-company-by-pitching-fake-screenshots-that-forced-them-to-turn-the-gba-into-a-3d-gaming-machine-nobody-could-believe-it/"
  ],
  "community_members_num": 47014773,
  "community_url": "https://www.reddit.com/r/gaming/",
  "community_description": "The Number One Gaming forum on the Internet.",
  "user_id": "t2_1kv65fo8oy",
  "comments": [
    { "comment": "This is the definition of 'Fake it till you make it'...", "user_commenting": "OLD-man87", "num_upvotes": 1 },
    { "comment": "THPS2 on the Gameboy Advance is one of my fondest childhood memories", "user_commenting": "HEAT-FS", "num_upvotes": 1 }
  ],
  "related_posts": ["... 25 related posts ..."],
  "timestamp": "2026-02-25T17:38:54.731Z",
  "input": { "url": "https://www.reddit.com/r/gaming/comments/1rdppqz/..." }
}
```

---

### 3.6 LinkedIn

**Source file:** [linkedin.json](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/data/linkedin.json) (trimmed — full file includes 5+ more_relevant_posts with massive embedded_links arrays)

```json
{
  "url": "https://www.linkedin.com/posts/poki_gamedev-webgaming-monetisation-activity-7414576322225778688-o4T2",
  "id": "7414576322225778688",
  "user_id": "poki",
  "title": "#gamedev #webgaming #monetisation #gamingindustry #poki #gamesindustry | Poki",
  "headline": "Our Head of Game Developer Operations & Partnerships Joep van Duinen recently wrote an article for Gamesforum...",
  "post_text": "Our Head of Game Developer Operations & Partnerships Joep van Duinen recently wrote an article for Gamesforum to discuss best practices for Rewarded Video monetization on web. With 100M+ monthly visits to Poki alone, there's a huge audience for developers to monetize. In his 2026 guide, Joep breaks down how top developers are reaching $1,000,000 in annual revenue. 🚀 Read his full guide below: https://lnkd.in/gHyCjY96 #GameDev #WebGaming #Monetisation #GamingIndustry #Poki #GamesIndustry",
  "date_posted": "2026-01-07T07:59:11.367Z",
  "hashtags": ["#GameDev", "#WebGaming", "#Monetisation", "#GamingIndustry", "#Poki", "#GamesIndustry"],
  "embedded_links": [
    "https://nl.linkedin.com/in/joep-van-duinen-3838661a?trk=public_post-text",
    "https://uk.linkedin.com/company/games-forum-ltd?trk=public_post-text",
    "https://lnkd.in/gHyCjY96",
    "https://www.linkedin.com/feed/hashtag/gamedev",
    "https://www.linkedin.com/feed/hashtag/webgaming",
    "https://www.linkedin.com/feed/hashtag/monetisation",
    "https://www.linkedin.com/feed/hashtag/gamingindustry",
    "https://www.linkedin.com/feed/hashtag/poki",
    "https://www.linkedin.com/feed/hashtag/gamesindustry"
  ],
  "images": ["https://media.licdn.com/dms/image/..."],
  "videos": null,
  "num_likes": 64,
  "num_comments": 2,
  "more_relevant_posts": ["... 5 related posts ..."],
  "timestamp": "2026-02-25T17:38:10.123Z",
  "input": { "url": "https://www.linkedin.com/posts/poki_..." }
}
```

---

### 3.7 Pinterest

**Source file:** [pinterest.json](file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/data/pinterest.json)

```json
{
  "url": "https://www.pinterest.com/pin/1110418851898928764/",
  "post_id": "1110418851898928764",
  "title": "Xbox Free Games",
  "content": "Xbox Games Showcase 2025 - Scheduled for today, June 8, featuring The Outer Worlds 2 Direct immediately after #xbox #XboxShowcase #gaming #XboxGamesShowcase",
  "date_posted": "2025-06-30T06:50:22.000Z",
  "user_name": "thegamingreroll",
  "user_url": "https://www.pinterest.com/thegamingreroll",
  "user_id": "1110418989282283681",
  "likes": 0,
  "attached_files": [
    "https://v1.pinimg.com/videos/iht/hls/.../668bb821d906ff95727c9d08ff164b08.m3u8",
    "https://i.pinimg.com/originals/5c/37/3a/5c373a29abbe0e8467d6c8de2257a4d6.jpg"
  ],
  "image_video_url": "https://i.pinimg.com/originals/5c/37/3a/5c373a29abbe0e8467d6c8de2257a4d6.jpg",
  "video_length": 8,
  "hashtags": [
    "Xbox Free Games", "Top Xbox Games", "Video Games 2025",
    "Original Xbox Backwards Compatible", "Best Video Games 2024",
    "Video Game Announcement", "Xbox Backwards Compatible Original Games",
    "Electronic Gaming Monthly", "Xbox Series X Games"
  ],
  "post_type": "video",
  "comments_num": 0,
  "ingredients_description": null,
  "timestamp": "2026-02-25T17:38:21.094Z",
  "input": { "url": "https://www.pinterest.com/pin/1110418851898928764/" }
}
```

---

## 4. Usable Fields for Verification

| Platform | Text Field | Hashtag Field | External URL Field | Likes Field |
|----------|-----------|--------------|-------------------|-------------|
| **Twitter** | `description` | `hashtags[]` | `external_url` | `likes` |
| **Instagram** | `description` | `hashtags[]` | ❌ (URLs in `description` text) | `likes` |
| **TikTok** | `description` | `hashtags[]` | ❌ (URLs in `description` text) | `digg_count` |
| **Facebook** | `content` | `hashtags[]` | `post_external_link` | `likes` |
| **Reddit** | `title` | ❌ (not supported by platform) | `embedded_links[]` | `num_upvotes` |
| **LinkedIn** | `post_text` | `hashtags[]` | `embedded_links[]` | `num_likes` |
| **Pinterest** | `content` | `hashtags[]` | ❌ (not exposed by BD) | `likes` |

---

## 5. Validation Method

A post is **Valid** if `error == null` AND at least **one** of these is true:

### Method 1: Hashtag Match

Check `hashtags[]` for `#kultgames` or `#kult.games` (case-insensitive).

Works on: Twitter, Instagram, TikTok, Facebook, LinkedIn, Pinterest

### Method 2: URL Match (dedicated field)

Check `external_url`, `post_external_link`, or `embedded_links[]` for `kult.games`.

Works on: Twitter (`external_url`), Facebook (`post_external_link`), Reddit (`embedded_links`), LinkedIn (`embedded_links`)

### Method 3: Regex on Text Content

Search the text field for `kult.games` URL or `@kultgames` mention using:

```
Regex: https?://[^\s]*kult\.games[^\s]*
```

Apply to: `description` (Twitter, Instagram, TikTok), `content` (Facebook, Pinterest), `post_text` (LinkedIn), `title` (Reddit)

### Combined Logic

```
is_valid = hashtag_match(hashtags[]) 
        OR url_match(external_url | post_external_link | embedded_links[])
        OR regex_match(description | content | post_text | title)
```

---

## 6. Scoring

```
Score = Likes (or platform equivalent)
```

| Platform | Likes Field | Sample Value |
|----------|-------------|-------------|
| Twitter | `likes` | 4 |
| Instagram | `likes` | 1,377 |
| TikTok | `digg_count` | 24,400 |
| Facebook | `likes` | 43 |
| Reddit | `num_upvotes` | 7,469 |
| LinkedIn | `num_likes` | 64 |
| Pinterest | `likes` | 0 |
