// src/upload/controller.rs
use axum::{extract::Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;
use utoipa::ToSchema;

use crate::config::CONFIG;
use crate::external::digital_ocean::spaces::SpacesService;
use crate::handler::{ApiResponse, AppError};
use crate::middleware::AuthPlayer;

#[derive(Deserialize, ToSchema)]
pub struct PresignRequest {
    pub filename: String,
    #[serde(alias = "contentType")]
    pub content_type: String,
}

#[derive(Serialize, ToSchema)]
pub struct PresignResponse {
    pub upload_url: String,
    pub public_url: String,
    pub required_headers: HashMap<String, String>,
}

#[utoipa::path(
    post,
    path = "/api/upload/presign",
    security(
        ("bearer_auth" = [])
    ),
    request_body = PresignRequest,
    responses(
        (status = 200, description = "Generated a presigned upload URL", body = crate::openapi::PresignApiResponse),
        (status = 400, description = "Invalid filename or unsupported content type", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Failed to generate upload URL", body = PresignResponse)
    ),
    tag = "Upload"
)]
#[instrument(skip(payload, auth))]
pub async fn generate_presigned_url(
    auth: AuthPlayer,
    Json(payload): Json<PresignRequest>,
) -> impl IntoResponse {
    let original_filename = payload.filename.trim();
    if original_filename.is_empty() {
        return AppError::BadRequest("filename is required".to_string()).into_response();
    }

    let content_type = payload.content_type.trim().to_ascii_lowercase();
    let extension = match allowed_extension_for_content_type(&content_type) {
        Some(extension) => extension,
        None => {
            return AppError::BadRequest(format!(
                "Unsupported content_type '{}'",
                payload.content_type
            ))
            .into_response();
        }
    };

    let service = SpacesService::new();
    let upload_path = CONFIG.do_spaces.effective_upload_path();
    let wallet_segment = safe_path_segment(&auth.wallet_address);
    let object_key = object_key_for_upload(&upload_path, &wallet_segment, extension);

    match service
        .generate_presigned_upload_url_for_key(&object_key)
        .await
    {
        Ok(presigned_request) => {
            tracing::info!(
                filename = %original_filename,
                content_type = %content_type,
                object_key = %object_key,
                "Generated presigned URL successfully"
            );

            // Extract headers required for the upload
            let mut required_headers = HashMap::new();
            for (name, value) in presigned_request.headers() {
                let val_str = value.to_string();
                required_headers.insert(name.to_string(), val_str);
            }

            let response_data = PresignResponse {
                upload_url: presigned_request.uri().to_string(),
                public_url: service.public_url_for_key(&object_key),
                required_headers,
            };

            ApiResponse::success(response_data).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate presigned URL");
            AppError::Internal("Failed to generate upload URL".to_string()).into_response()
        }
    }
}

fn allowed_extension_for_content_type(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "video/mp4" => Some("mp4"),
        "video/webm" => Some("webm"),
        _ => None,
    }
}

fn safe_path_segment(value: &str) -> String {
    let segment: String = value
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect();

    if segment.is_empty() {
        "unknown-wallet".to_string()
    } else {
        segment
    }
}

fn object_key_for_upload(upload_path: &str, wallet_segment: &str, extension: &str) -> String {
    format!(
        "{}/{}/{}.{}",
        upload_path.trim().trim_matches('/'),
        wallet_segment,
        nanoid::nanoid!(),
        extension
    )
}

#[cfg(test)]
mod tests {
    use super::{
        allowed_extension_for_content_type, object_key_for_upload, safe_path_segment,
        PresignRequest,
    };

    #[test]
    fn maps_all_supported_content_types_to_safe_extensions() {
        let cases = [
            ("image/jpeg", Some("jpg")),
            ("image/jpg", Some("jpg")),
            ("image/png", Some("png")),
            ("image/gif", Some("gif")),
            ("image/webp", Some("webp")),
            ("video/mp4", Some("mp4")),
            ("video/webm", Some("webm")),
        ];

        for (content_type, expected_extension) in cases {
            assert_eq!(
                allowed_extension_for_content_type(content_type),
                expected_extension
            );
        }
    }

    #[test]
    fn rejects_unsupported_content_types() {
        for content_type in [
            "",
            "text/plain",
            "application/json",
            "image/svg+xml",
            "application/octet-stream",
            "video/quicktime",
        ] {
            assert_eq!(allowed_extension_for_content_type(content_type), None);
        }
    }

    #[test]
    fn safe_path_segment_preserves_wallet_casing_and_removes_path_chars() {
        assert_eq!(safe_path_segment(" 0xAbC123 "), "0xAbC123");
        assert_eq!(safe_path_segment("../0xAbC/123?x=1"), "0xAbC123x1");
    }

    #[test]
    fn safe_path_segment_falls_back_when_empty() {
        assert_eq!(safe_path_segment(" ../ "), "unknown-wallet");
    }

    #[test]
    fn object_key_uses_server_generated_name_under_wallet_path() {
        let key = object_key_for_upload("/moments/", "0xAbC123", "png");

        assert!(key.starts_with("moments/0xAbC123/"));
        assert!(key.ends_with(".png"));
        assert!(!key.contains("original-file-name"));
        assert_eq!(key.matches('/').count(), 2);
    }

    #[test]
    fn presign_request_accepts_legacy_and_camel_case_content_type() {
        let snake: PresignRequest = serde_json::from_value(serde_json::json!({
            "filename": "clip.png",
            "content_type": "image/png"
        }))
        .expect("snake_case content_type should deserialize");

        let camel: PresignRequest = serde_json::from_value(serde_json::json!({
            "filename": "clip.png",
            "contentType": "image/png"
        }))
        .expect("camelCase contentType should deserialize");

        assert_eq!(snake.content_type, "image/png");
        assert_eq!(camel.content_type, "image/png");
    }
}
