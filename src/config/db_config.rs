// src/config/db_config.rs
// Database configuration (MongoDB settings and collection names)

use std::env;

/// Database configuration for MongoDB
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// MongoDB connection URI (required)
    pub mongo_uri: String,
    /// MongoDB database name (required)
    pub mongo_db_name: String,
    /// Connection retry attempts (default: 5)
    pub mongo_conn_retries: u32,

    // Collection names
    /// Games collection name (default: kultbrowser_games)
    pub games_collection: String,
    /// Content configs collection name (default: kultbrowser_content_configs)
    pub content_collection: String,
    /// Campaigns collection name (default: kultbrowser_campaigns)
    pub campaigns_collection: String,
    /// Chatbot knowledge collection name (default: kultbrowser_chatbot_knowledge)
    pub chatbot_collection: String,
}

impl DbConfig {
    /// Load database config from environment variables.
    /// Panics with clear error message if required vars are missing.
    pub fn from_env() -> Self {
        let mongo_uri = env::var("MONGO_URI")
            .unwrap_or_else(|_| panic!("❌ MONGO_URI is required. Set it in .env or environment."));

        let mongo_db_name = env::var("MONGO_DB_NAME").unwrap_or_else(|_| {
            panic!("❌ MONGO_DB_NAME is required. Set it in .env or environment.")
        });

        Self {
            mongo_uri,
            mongo_db_name,

            mongo_conn_retries: env::var("MONGO_CONN_RETRIES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("MONGO_CONN_RETRIES must be a valid u32"),

            // Collection names with defaults
            games_collection: env::var("GAMES_MONGO_COLL_NAME")
                .unwrap_or_else(|_| "kultbrowser_games".to_string()),

            content_collection: env::var("CONTENT_MONGO_COLL_NAME")
                .unwrap_or_else(|_| "kultbrowser_content_configs".to_string()),

            campaigns_collection: env::var("CAMPAIGNS_MONGO_COLL_NAME")
                .unwrap_or_else(|_| "kultbrowser_campaigns".to_string()),

            chatbot_collection: env::var("CHATBOT_MONGO_COLL_NAME")
                .unwrap_or_else(|_| "kultbrowser_chatbot_knowledge".to_string()),
        }
    }
}
