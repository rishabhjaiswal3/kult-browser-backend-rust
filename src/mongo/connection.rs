use mongodb::{Client, Database};
use std::env;
use std::time::Duration;
use tokio::time::sleep;

pub async fn connect() -> Result<Database, mongodb::error::Error> {
    dotenvy::dotenv().ok();

    let mongo_uri = env::var("MONGO_URI").unwrap_or_else(|_| "".to_string());
    let mongo_db_name = env::var("MONGO_DB_NAME").unwrap_or_else(|_| "".to_string());
    let mongo_conn_retries: u32 = env::var("MONGO_CONN_RETRIES").unwrap_or_else(|_| "".to_string()).parse().unwrap_or(5);

    let mut last_error = None;

    for attempt in 1..=mongo_conn_retries {

        match Client::with_uri_str(&mongo_uri).await {
            Ok(client) => {
                match client
                    .database(&mongo_db_name)
                    .run_command(mongodb::bson::doc! { "ping": 1 })
                    .await
                {
                    Ok(_) => {
                        println!("MongoDB connected to: {}", mongo_db_name);
                        return Ok(client.database(&mongo_db_name));
                    }
                    Err(e) => {
                        println!("Ping failed: {}", e);
                        last_error = Some(e);
                    }
                }
            }
            Err(e) => {
                println!("Connection failed: {}", e);
                last_error = Some(e);
            }
        }

        if attempt < mongo_conn_retries {
            println!("Retrying connection...");
            sleep(Duration::from_secs(1)).await;
        }
    }

    Err(last_error.expect("No error captured"))
}