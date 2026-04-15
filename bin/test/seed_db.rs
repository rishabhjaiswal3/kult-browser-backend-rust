use mongodb::{
    bson::{doc, DateTime},
    Client, Database,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Seeding database directly...");
    dotenvy::dotenv().ok();

    let mongo_uri =
        env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    // db_name: kult_browser is what the server uses.
    let db_name = env::var("MONGO_DB_NAME").unwrap_or_else(|_| "kult_browser".to_string());

    let client = Client::with_uri_str(&mongo_uri).await?;
    let db: Database = client.database(&db_name);
    let collection = db.collection::<mongodb::bson::Document>("moments");

    let wallet = "0x1234567890123456789012345678901234567890";
    let now = DateTime::now();

    let moments = vec![
        doc! {
            "momentId": "crypto-cosmos",
            "playerWalletAddress": wallet,
            "title": "Crypto Universe",
            "description": "Analyzing the cosmos of crypto.",
            "tags": ["crypto", "cosmos", "gif"],
            "assetUrl": "https://kult-browser.sfo3.digitaloceanspaces.com/moments/crypto-crypto-cosmos.gif",
            "createdAt": now,
            "updatedAt": now
        },
        doc! {
            "momentId": "sample-view",
            "playerWalletAddress": wallet,
            "title": "Sample View",
            "description": "Just a sample.",
            "tags": ["sample"],
            "assetUrl": "https://kult-browser.sfo3.digitaloceanspaces.com/moments/68747470733a2f2f796176757a63656c696b65722e6769746875622e696f2f73616d706c652d696d616765732f696d6167652d313032312e6a7067.jpeg",
            "createdAt": now,
            "updatedAt": now
        },
        doc! {
            "momentId": "cool-download",
            "playerWalletAddress": wallet,
            "title": "Cool Download",
            "description": "Downloading.",
            "tags": ["download"],
            "assetUrl": "https://kult-browser.sfo3.digitaloceanspaces.com/moments/download.jpeg",
            "createdAt": now,
            "updatedAt": now
        },
    ];

    for moment in moments {
        let filter = doc! { "momentId": moment.get_str("momentId")? };
        let opts = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let update = doc! { "$set": moment };
        collection
            .update_one(filter, update)
            .with_options(opts)
            .await?;
        println!("Upserted moment");
    }

    println!("Done seeding!");
    Ok(())
}
