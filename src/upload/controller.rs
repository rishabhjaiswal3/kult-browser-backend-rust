// src/upload/controller.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;
use utoipa::ToSchema;

use crate::external::digital_ocean::spaces::SpacesService;

#[derive(Deserialize, ToSchema)]
pub struct PresignRequest {
    pub filename: String,
    pub content_type: String,
}

#[derive(Serialize, ToSchema)]
pub struct PresignResponse {
    pub upload_url: String,
    pub public_url: String,
    pub required_headers: HashMap<String, String>,
}

use crate::middleware::AuthPlayer;

#[utoipa::path(
    post,
    path = "/api/upload/presign",
    security(
        ("bearer_auth" = [])
    ),
    request_body = PresignRequest,
    responses(
        (status = 200, description = "Generated a presigned upload URL", body = crate::openapi::PresignApiResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Failed to generate upload URL", body = PresignResponse)
    ),
    tag = "Upload"
)]
#[instrument(skip(payload, _auth))]
pub async fn generate_presigned_url(
    _auth: AuthPlayer,
    Json(payload): Json<PresignRequest>,
) -> impl IntoResponse {
    let service = SpacesService::new();

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

            let upload_path = config.effective_upload_path();
            let key = format!("{}/{}", upload_path, payload.filename);

            let public_url = format!("https://{}.{}/{}", config.bucket, endpoint_clean, key);

            // Extract headers required for the upload
            let mut required_headers = HashMap::new();
            for (name, value) in presigned_request.headers() {
                let val_str = value.to_string();
                required_headers.insert(name.to_string(), val_str);
            }

            use crate::handler::ApiResponse;

            let response_data = PresignResponse {
                upload_url: presigned_request.uri().to_string(),
                public_url,
                required_headers,
            };

            ApiResponse::success(response_data).into_response()
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
                .into_response()
        }
    }
}
