use crate::content::model::ContentConfig;
use mongodb::{Collection, Database};

pub struct ContentConfigRepository {
    collection: Collection<ContentConfig>,
}

impl ContentConfigRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.collection::<ContentConfig>("content_configs");
        Self { collection }
    }

    pub async fn find_config(&self, page: &str, section: &str) -> Option<ContentConfig> {
        self.collection
            .find_one(mongodb::bson::doc! { "page": page, "section": section })
            .await
            .ok()
            .flatten()
    }

    // Admin support
    pub async fn upsert(&self, config: &ContentConfig) -> mongodb::error::Result<()> {
        let options = mongodb::options::FindOneAndReplaceOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .find_one_and_replace(
                mongodb::bson::doc! { "page": &config.page, "section": &config.section },
                config,
            )
            .with_options(options)
            .await?;

        Ok(())
    }
}
