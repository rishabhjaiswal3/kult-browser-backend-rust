// src/external/digital_ocean/spaces/service.rs

use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use std::time::Duration;

use crate::config::CONFIG;

#[derive(Clone)]
pub struct SpacesService {
    client: Client,
    bucket: String,
}

impl SpacesService {
    /// Create a new SpacesService instance using global configuration
    pub fn new() -> Self {
        let config = &CONFIG.do_spaces;

        // Create static credentials
        let credentials =
            Credentials::new(&config.access_key, &config.secret_key, None, None, "static");

        // Configure S3 client
        let s3_config = aws_sdk_s3::Config::builder()
            .credentials_provider(credentials)
            .region(Region::new(config.region.clone()))
            .endpoint_url(&config.endpoint)
            .force_path_style(true) // Spaces supports path style too, might be safer for presigned
            .behavior_version(BehaviorVersion::latest())
            .build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket: config.bucket.clone(),
        }
    }

    /// Generate a presigned URL for uploading a file (PUT)
    pub async fn generate_presigned_upload_url_for_key(
        &self,
        object_key: &str,
    ) -> Result<aws_sdk_s3::presigning::PresignedRequest, String> {
        let config = &CONFIG.do_spaces;
        let expiration = Duration::from_secs(config.presigned_expiration);

        // Create presigning config
        let presigning_config = PresigningConfig::builder()
            .expires_in(expiration)
            .build()
            .map_err(|e| format!("Failed to create presigning config: {}", e))?;

        // Create presigned request
        let presigned_request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(object_key)
            // Make uploaded objects publicly readable.
            // This is signed into the URL as x-amz-acl and must be sent by the client.
            .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
            // Do not sign content-type for browser uploads.
            // Browser/runtime header normalization can produce casing/value variations
            // that otherwise trigger SignatureDoesNotMatch on Spaces.
            .presigned(presigning_config)
            .await
            .map_err(|e| format!("Failed to generate presigned URL: {}", e))?;

        Ok(presigned_request)
    }

    pub fn public_url_for_key(&self, object_key: &str) -> String {
        let endpoint_clean = CONFIG
            .do_spaces
            .endpoint
            .trim_start_matches("https://")
            .trim_start_matches("http://");

        format!("https://{}.{}/{}", self.bucket, endpoint_clean, object_key)
    }

    /// Check if a file exists in the bucket by its public URL
    pub async fn check_file_exists(&self, public_url: &str) -> bool {
        // Parse key from URL
        // Expected format: https://<bucket>.<endpoint>/<key>

        let config = &CONFIG.do_spaces;
        let endpoint_host = config
            .endpoint
            .trim_start_matches("https://")
            .trim_start_matches("http://");

        // Case 1: Virtual hosted style: https://bucket.endpoint/key
        // Case 2: Path style: https://endpoint/bucket/key

        // Check path style first (e.g. sfo3.digitaloceanspaces.com/kult-browser/...)
        let path_prefix = format!("{}/{}/", endpoint_host, self.bucket);
        let key = if public_url.contains(&path_prefix) {
            public_url
                .split(&path_prefix)
                .nth(1)
                .unwrap_or("")
                .to_string()
        } else {
            // Fallback to virtual hosted or simple extraction
            let domain = format!("{}.{}", self.bucket, endpoint_host);
            if public_url.contains(&domain) {
                public_url
                    .split(&domain)
                    .nth(1)
                    .unwrap_or("")
                    .trim_start_matches('/')
                    .to_string()
            } else {
                public_url.split('/').last().unwrap_or("").to_string()
            }
        };

        if key.is_empty() {
            return false;
        }

        let result = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await;

        match result {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
