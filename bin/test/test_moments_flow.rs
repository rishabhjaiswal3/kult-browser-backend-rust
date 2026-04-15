use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

const API_URL: &str = "http://localhost:4000/api";
const TEST_WALLET: &str = "0x1234567890123456789012345678901234567890";
const TEST_FILE_NAME: &str = "test_moment_flow.txt";
const TEST_FILE_CONTENT: &str = "This is a test file for the full moments flow.";

#[derive(Deserialize, Debug)]
struct ApiResponse<T> {
    data: T,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: String,
}

#[derive(Deserialize, Debug)]
struct CreateMomentResponse {
    #[serde(rename = "momentId")]
    moment_id: String,
}

#[derive(Deserialize, Debug)]
struct PresignResponse {
    upload_url: String,
    public_url: String,
    required_headers: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    println!("🚀 Starting End-to-End Moments Flow Test\n");

    // 1. Create a dummy file
    tokio::fs::write(TEST_FILE_NAME, TEST_FILE_CONTENT).await?;
    println!("✅ 1. Created local test file: {}", TEST_FILE_NAME);

    // 2. Login to get token
    println!("🔄 2. Logging in as {}...", TEST_WALLET);
    let login_res = client
        .post(format!("{}/player/login", API_URL))
        .json(&json!({
            "walletAddress": TEST_WALLET
        }))
        .send()
        .await?;

    if !login_res.status().is_success() {
        let error = login_res.text().await?;
        return Err(format!("Login failed: {}", error).into());
    }

    let login_data: ApiResponse<LoginResponse> = login_res.json().await?;
    let token = login_data.data.token;
    println!("✅ Logged in! Token received.");

    // 3. Create Moment (Draft - no assetUrl yet)
    println!("\n🔄 3. Creating Moment Draft...");
    let create_res = client
        .post(format!("{}/moments/register", API_URL))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "title": "Test Moment Flow",
            "description": "Integration test for upload flow",
            "tags": ["test", "upload"]
            // No assetUrl here
        }))
        .send()
        .await?;

    if !create_res.status().is_success() {
        let error = create_res.text().await?;
        return Err(format!("Create Moment failed: {}", error).into());
    }

    let create_data: ApiResponse<CreateMomentResponse> = create_res.json().await?;
    let moment_id = create_data.data.moment_id;
    println!("✅ Moment Created! ID: {}", moment_id);

    // 4. Get Presigned URL
    println!("\n🔄 4. Requesting Presigned URL...");
    let presign_res = client
        .post(format!("{}/upload/presign", API_URL))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "filename": TEST_FILE_NAME,
            "content_type": "text/plain"
        }))
        .send()
        .await?;

    if !presign_res.status().is_success() {
        let error = presign_res.text().await?;
        return Err(format!("Presign failed: {}", error).into());
    }

    let presign_data: ApiResponse<PresignResponse> = presign_res.json().await?;
    let presign_info = presign_data.data;
    println!("✅ Got Upload URL: {}", presign_info.upload_url);
    println!("Expected Public URL: {}", presign_info.public_url);

    // 5. Upload File to DO Spaces
    println!("\n🔄 5. Uploading file to DigitalOcean Spaces...");
    let file_content = tokio::fs::read(TEST_FILE_NAME).await?;

    let mut upload_req = client.put(&presign_info.upload_url).body(file_content);

    // Add required headers
    for (key, value) in presign_info.required_headers {
        upload_req = upload_req.header(&key, value);
    }

    let upload_res = upload_req.send().await?;

    if !upload_res.status().is_success() {
        let error = upload_res.text().await?;
        return Err(format!("Upload failed: {}", error).into());
    }
    println!("✅ Upload Successful!");

    // 6. Update Moment with assetUrl
    println!("\n🔄 6. Updating Moment with Asset URL...");
    let update_res = client
        .patch(format!("{}/moments/{}", API_URL, moment_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "assetUrl": presign_info.public_url
        }))
        .send()
        .await?;

    if !update_res.status().is_success() {
        let error = update_res.text().await?;
        return Err(format!("Update Moment failed (Backend Verification): {}", error).into());
    }

    println!("✅ Update Successful! Backend verified file existence.");

    // Cleanup
    tokio::fs::remove_file(TEST_FILE_NAME).await?;
    println!("\n🎉 Full Flow Verified Successfully!");

    Ok(())
}
