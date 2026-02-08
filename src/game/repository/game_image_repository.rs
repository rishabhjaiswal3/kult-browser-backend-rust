use chrono::Utc;
use mongodb::{
    bson::{doc, to_bson},
    options::FindOneAndUpdateOptions,
    Collection, Database,
};

use crate::config::CONFIG;
use crate::game::model::game_image_model::{
    GameImages, ImageObject, IndexedImage, OrientedImage, OrientedImageArray,
};
use crate::game::model::game_model::GameModel;

pub struct GameModelImageRepository {
    collection: Collection<GameModel>,
}

impl GameModelImageRepository {
    // Create a new repository instance
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<GameModel>(&CONFIG.db.games_collection),
        }
    }

    // ─────────────────────────────────────────────────────────────
    // HERO IMAGES
    // ─────────────────────────────────────────────────────────────

    // Update entire hero images
    pub async fn update_hero(
        &self,
        identification: &str,
        hero: &OrientedImage,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        let hero_bson = to_bson(hero)
            .map_err(|e| mongodb::error::Error::custom(format!("Serialization error: {}", e)))?;

        let update_doc = doc! {
            "$set": {
                "images.hero": hero_bson,
                "updated_at": Utc::now().to_rfc3339()
            }
        };

        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "identification": identification }, update_doc)
            .with_options(options)
            .await
    }

    // Update hero image for specific orientation
    pub async fn update_hero_orientation(
        &self,
        identification: &str,
        orientation: &str, // "horizontal", "vertical", "square", "ultrawide"
        image: &ImageObject,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        let image_bson = to_bson(image)
            .map_err(|e| mongodb::error::Error::custom(format!("Serialization error: {}", e)))?;

        let field = format!("images.hero.{}", orientation);
        let update_doc = doc! {
            "$set": {
                &field: image_bson,
                "updated_at": Utc::now().to_rfc3339()
            }
        };

        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "identification": identification }, update_doc)
            .with_options(options)
            .await
    }

    // ─────────────────────────────────────────────────────────────
    // CAROUSEL IMAGES
    // ─────────────────────────────────────────────────────────────

    // Update entire carousel
    pub async fn update_carousel(
        &self,
        identification: &str,
        carousel: &OrientedImageArray,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        let carousel_bson = to_bson(carousel)
            .map_err(|e| mongodb::error::Error::custom(format!("Serialization error: {}", e)))?;

        let update_doc = doc! {
            "$set": {
                "images.carousel": carousel_bson,
                "updated_at": Utc::now().to_rfc3339()
            }
        };

        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "identification": identification }, update_doc)
            .with_options(options)
            .await
    }

    // Update carousel for specific orientation
    pub async fn update_carousel_orientation(
        &self,
        identification: &str,
        orientation: &str,
        images: &Vec<IndexedImage>,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        let images_bson = to_bson(images)
            .map_err(|e| mongodb::error::Error::custom(format!("Serialization error: {}", e)))?;

        let field = format!("images.carousel.{}", orientation);
        let update_doc = doc! {
            "$set": {
                &field: images_bson,
                "updated_at": Utc::now().to_rfc3339()
            }
        };

        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "identification": identification }, update_doc)
            .with_options(options)
            .await
    }

    // Add single image to carousel
    pub async fn add_carousel_image(
        &self,
        identification: &str,
        orientation: &str,
        image: &IndexedImage,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        let image_bson = to_bson(image)
            .map_err(|e| mongodb::error::Error::custom(format!("Serialization error: {}", e)))?;

        let field = format!("images.carousel.{}", orientation);
        let update_doc = doc! {
            "$push": {
                &field: image_bson
            },
            "$set": {
                "updated_at": Utc::now().to_rfc3339()
            }
        };

        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "identification": identification }, update_doc)
            .with_options(options)
            .await
    }

    // Remove image from carousel by index
    pub async fn remove_carousel_image(
        &self,
        identification: &str,
        orientation: &str,
        index: u32,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        let field = format!("images.carousel.{}", orientation);

        // MongoDB $pull to remove by index field
        let update_doc = doc! {
            "$pull": {
                &field: { "index": index }
            },
            "$set": {
                "updated_at": Utc::now().to_rfc3339()
            }
        };

        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "identification": identification }, update_doc)
            .with_options(options)
            .await
    }

    // ─────────────────────────────────────────────────────────────
    // FULL IMAGES OBJECT
    // ─────────────────────────────────────────────────────────────

    // Update entire images object
    pub async fn update_all_images(
        &self,
        identification: &str,
        images: &GameImages,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        let images_bson = to_bson(images)
            .map_err(|e| mongodb::error::Error::custom(format!("Serialization error: {}", e)))?;

        let update_doc = doc! {
            "$set": {
                "images": images_bson,
                "updated_at": Utc::now().to_rfc3339()
            }
        };

        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "identification": identification }, update_doc)
            .with_options(options)
            .await
    }

    // Get images for a game
    pub async fn get_images(
        &self,
        identification: &str,
    ) -> Result<Option<GameImages>, mongodb::error::Error> {
        let game = self
            .collection
            .find_one(doc! { "identification": identification })
            .await?;

        Ok(game.map(|g| g.images))
    }
}
