use crate::content::model::ContentConfig;
use crate::handler::AppError;
use mongodb::{Collection, Database};

pub struct ContentConfigRepository {
    collection: Collection<ContentConfig>,
}

impl ContentConfigRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.collection::<ContentConfig>("content_configs");
        Self { collection }
    }

    /// Find a content config by page and section.
    pub async fn find_config(&self, page: &str, section: &str) -> Result<ContentConfig, AppError> {
        self.collection
            .find_one(mongodb::bson::doc! { "page": page, "section": section })
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Section '{}' not found on page '{}'",
                    section, page
                ))
            })
    }

    pub async fn upsert(&self, config: &ContentConfig) -> Result<(), AppError> {
        let options = mongodb::options::FindOneAndReplaceOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .find_one_and_replace(
                mongodb::bson::doc! { "page": &config.page, "section": &config.section },
                config,
            )
            .with_options(options)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }
}
