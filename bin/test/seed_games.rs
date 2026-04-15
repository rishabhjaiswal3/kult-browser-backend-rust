use futures::stream::{self, StreamExt};
use kult_browser_backend_rust::{
    game::{
        model::{
            game_image_model::{
                GameImages, ImageObject, IndexedImage, OrientedImage, OrientedImageArray,
            },
            game_model::GameModel,
            util,
        },
        repository::GameModelRepository,
    },
    mongo::connection,
};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// ─────────────────────────────────────────────────────────────
// LEGACY DATA STRUCTURES (Mapping Source JSON)
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct LegacyGame {
    #[serde(rename = "_id")]
    id: LegacyId,
    identification: String,
    name: String,
    slogan: Option<String>,
    #[serde(rename = "type")]
    game_type: String, // "browser_only", "desktop_only"
    url: String,
    about: Option<serde_json::Value>, // Polymorphic: Array or Object
    tags: Option<Vec<String>>,
    images: LegacyGameImages,
    home: Option<serde_json::Value>,
    metadata: Option<serde_json::Value>,
    #[serde(rename = "createdAt")]
    created_at: Option<LegacyDate>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<LegacyDate>,
    chain: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyId {
    #[serde(rename = "$oid")]
    oid: String,
}

#[derive(Debug, Deserialize)]
struct LegacyDate {
    #[serde(rename = "$date")]
    date: String,
}

#[derive(Debug, Deserialize)]
struct LegacyGameImages {
    hero: LegacyOrientedImage,
    carousel: LegacyOrientedImageArray,
}

#[derive(Debug, Deserialize)]
struct LegacyOrientedImage {
    horizontal: LegacyImageObject,
    vertical: Option<LegacyImageObject>,
}

#[derive(Debug, Deserialize)]
struct LegacyOrientedImageArray {
    horizontal: Vec<LegacyIndexedImage>,
    vertical: Vec<LegacyIndexedImage>,
}

#[derive(Debug, Deserialize)]
struct LegacyIndexedImage {
    index: u32,
    #[serde(flatten)]
    image: LegacyImageObject,
}

#[derive(Debug, Deserialize)]
struct LegacyImageObject {
    url: String,
    // Source JSON has these fields
    #[serde(rename = "size_in_KB")]
    size_in_kb: Option<f64>, // Source is float sometimes
    width: Option<u32>,
    height: Option<u32>,
    #[serde(rename = "type")]
    mime_type: Option<String>,
    name: Option<String>,
}

// ─────────────────────────────────────────────────────────────
// CONVERSION LOGIC
// ─────────────────────────────────────────────────────────────

impl From<LegacyGame> for GameModel {
    fn from(legacy: LegacyGame) -> Self {
        let platform = match legacy.game_type.as_str() {
            "browser_only" => "web",
            "desktop_only" => "desktop",
            _ => "web",
        }
        .to_string();

        let mut metadata = legacy.metadata.clone().unwrap_or(serde_json::json!({}));

        // Preserve extra fields in metadata
        if let Some(home) = legacy.home {
            metadata["home"] = home;
        }
        if let Some(chain) = legacy.chain {
            metadata["chain"] = serde_json::Value::String(chain);
        }
        // If about is complex/images, store in metadata.legacy_about
        let about_str = if let Some(ref about_val) = legacy.about {
            if about_val.is_array() {
                // It's text content, try to serialize or just keep generic
                Some("See description in metadata".to_string())
            } else {
                // It's likely the old image-based about
                metadata["legacy_about"] = about_val.clone();
                None
            }
        } else {
            None
        };

        // Convert timestamps
        let created_at = legacy
            .created_at
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d.date).ok())
            .map(|d| d.with_timezone(&chrono::Utc));
        let updated_at = legacy
            .updated_at
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d.date).ok())
            .map(|d| d.with_timezone(&chrono::Utc));

        let oid = ObjectId::parse_str(&legacy.id.oid).unwrap_or_else(|_| ObjectId::new());

        GameModel {
            id: oid,
            identification: legacy.identification,
            name: util::create_localized_with_cn(legacy.name),
            platform,
            url: legacy.url,
            images: convert_images(legacy.images),
            slogan: legacy.slogan.map(util::create_localized_with_cn),
            about: about_str.map(util::create_localized_with_cn),
            category: None, // Not in source
            tags: legacy.tags,
            rating: None,
            rating_count: None,
            metadata: Some(metadata),
            created_at,
            updated_at,
        }
    }
}

fn convert_images(legacy: LegacyGameImages) -> GameImages {
    GameImages {
        hero: OrientedImage {
            horizontal: util::create_localized(convert_image_obj(legacy.hero.horizontal)),
            vertical: util::create_localized(
                legacy
                    .hero
                    .vertical
                    .map(convert_image_obj)
                    .unwrap_or_default(),
            ),
            square: None,
            ultrawide: None,
        },
        carousel: OrientedImageArray {
            horizontal: legacy
                .carousel
                .horizontal
                .into_iter()
                .map(convert_indexed_image)
                .collect(),
            vertical: legacy
                .carousel
                .vertical
                .into_iter()
                .map(convert_indexed_image)
                .collect(),
            square: None,
            ultrawide: None,
        },
        thumbnail: None,
        icon: None,
        logo: None,
        screenshots: None,
    }
}

fn convert_indexed_image(legacy: LegacyIndexedImage) -> IndexedImage {
    IndexedImage {
        index: legacy.index,
        image: convert_image_obj(legacy.image),
    }
}

fn convert_image_obj(legacy: LegacyImageObject) -> ImageObject {
    ImageObject {
        url: legacy.url,
        width: legacy.width,
        height: legacy.height,
        alt: legacy.name.map(util::create_localized_with_cn), // Map name to alt
        size_in_kb: legacy.size_in_kb.map(|f| f as u32),
        mime_type: legacy.mime_type,
        blurhash: None,
        svg_content: None,
    }
}

// ─────────────────────────────────────────────────────────────
// MAIN EXECUTION
// ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // 1. Connect DB
    let db = connection::connect()
        .await
        .expect("Failed to connect to Mongo");
    let repo = GameModelRepository::new(&db);

    // 2. Read JSON File
    // Adjust path as needed, assuming running from project root
    let json_path = "../kult-browser-backend/Data/store_global.store_games.json";
    let path = Path::new(json_path);

    if !path.exists() {
        eprintln!("Error: File not found at {:?}", path.canonicalize());
        return;
    }

    let file = File::open(path).expect("Failed to open JSON file");
    let reader = BufReader::new(file);

    // 3. Parse JSON
    let legacy_games: Vec<LegacyGame> =
        serde_json::from_reader(reader).expect("Failed to parse JSON");

    println!("Found {} games to migrate.", legacy_games.len());

    // 4. Process & Insert
    let results = stream::iter(legacy_games)
        .map(|legacy| {
            let repo = &repo; // Capture reference
            async move {
                let identification = legacy.identification.clone();
                let new_game: GameModel = legacy.into();

                // Use replace to update existing or insert new
                match repo.replace(&identification, &new_game).await {
                    Ok(_) => {
                        println!("✅ Migrated: {}", identification);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to migrate {}: {}", identification, e);
                        Err(e)
                    }
                }
            }
        })
        .buffer_unordered(5) // Process 5 concurrently
        .collect::<Vec<_>>()
        .await;

    let success_count = results.iter().filter(|r| r.is_ok()).count();
    println!(
        "Migration Complete. Success: {}/{}",
        success_count,
        results.len()
    );
}
