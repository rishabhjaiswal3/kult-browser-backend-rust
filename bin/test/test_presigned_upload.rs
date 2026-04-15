use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

#[derive(Deserialize, Debug)]
struct PresignResponse {
    upload_url: String,
    public_url: String,
    required_headers: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // 1. Create a dummy file locally
    let filename = "test_presign.txt";
    let content = "Hello from Kult Browser Backend Test!";

    {
        let mut file = File::create(filename)?;
        file.write_all(content.as_bytes())?;
    }

    println!("Created test file: {}", filename);

    // 2. Request presigned URL from backend
    let url = "http://localhost:4000/api/upload/presign";
    println!("Requesting presign URL from: {}", url);

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "filename": filename,
            "content_type": "text/plain"
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        eprintln!("Failed to get presigned URL: {}", resp.status());
        let text = resp.text().await?;
        eprintln!("Response: {}", text);
        return Ok(());
    }

    let presign_data: PresignResponse = resp.json().await?;
    println!("Got presigned URL: {}", presign_data.upload_url);
    println!("Expected Public URL: {}", presign_data.public_url);
    println!("Required Headers: {:?}", presign_data.required_headers);

    // 3. Upload file directly to DO Spaces using the presigned URL
    println!("\nUploading file directly to DigitalOcean Spaces...");

    // We need to read the file content to upload it
    let file_content = std::fs::read(filename)?;

    let mut request = client.put(&presign_data.upload_url).body(file_content);

    // Add required headers (e.g., x-amz-acl)
    for (key, value) in &presign_data.required_headers {
        request = request.header(key, value);
    }

    let upload_resp = request.send().await?;

    if upload_resp.status().is_success() {
        println!("\n✅ Successfully uploaded file!");
        println!("You can verify by opening this URL in your browser:");
        println!("{}", presign_data.public_url);
    } else {
        eprintln!("\n❌ Failed to upload file: {}", upload_resp.status());
        let text = upload_resp.text().await?;
        eprintln!("Response: {}", text);
    }

    // Cleanup local file
    std::fs::remove_file(filename)?;

    Ok(())
}
