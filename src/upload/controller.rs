// src/upload/controller.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;

use crate::external::digital_ocean::spaces::SpacesService;

#[derive(Deserialize)]
pub struct PresignRequest {
    pub filename: String,
    pub content_type: String,
}

#[derive(Serialize)]
pub struct PresignResponse {
    pub upload_url: String,
    pub public_url: String,
    pub required_headers: HashMap<String, String>,
}

#[instrument(skip(payload))]
pub async fn generate_presigned_url(Json(payload): Json<PresignRequest>) -> impl IntoResponse {
    let service = SpacesService::new().await;

    match service
        .generate_presigned_upload_url(&payload.filename, &payload.content_type)
        .await
    {
        Ok(presigned_request) => {
            tracing::info!(
                filename = %payload.filename,
                "Generated presigned URL successfully"
            );

            // Construct public URL (assuming public read access)
            // Format: https://<bucket>.<endpoint>/<filename>
            // Note: endpoint in config includes https://, so we need to be careful
            let config = &crate::config::CONFIG.do_spaces;
            // Config endpoint: https://nyc3.digitaloceanspaces.com
            // Desired: https://<bucket>.nyc3.digitaloceanspaces.com/<filename>

            let endpoint_clean = config
                .endpoint
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            let public_url = format!(
                "https://{}.{}/{}",
                config.bucket, endpoint_clean, payload.filename
            );

            // Extract headers required for the upload
            let mut required_headers = HashMap::new();
            for (name, value) in presigned_request.headers() {
                let val_str = value.to_string();
                required_headers.insert(name.to_string(), val_str);
            }

            (
                StatusCode::OK,
                Json(PresignResponse {
                    upload_url: presigned_request.uri().to_string(),
                    public_url,
                    required_headers,
                }),
            )
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate presigned URL");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PresignResponse {
                    upload_url: "".to_string(),
                    public_url: "".to_string(),
                    required_headers: HashMap::new(),
                }),
            )
        }
    }
}
