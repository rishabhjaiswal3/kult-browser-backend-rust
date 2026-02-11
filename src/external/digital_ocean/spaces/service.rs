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
    pub async fn new() -> Self {
        let config = &CONFIG.do_spaces;

        // Create static credentials
        let credentials =
            Credentials::new(&config.access_key, &config.secret_key, None, None, "static");

        // Configure S3 client
        let s3_config = aws_sdk_s3::Config::builder()
            .credentials_provider(credentials)
            .region(Region::new(config.region.clone()))
            .endpoint_url(&config.endpoint)
            .force_path_style(false) // Spaces supports virtual-hosted style
            .behavior_version(BehaviorVersion::latest())
            .build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket: config.bucket.clone(),
        }
    }

    /// Generate a presigned URL for uploading a file (PUT)
    pub async fn generate_presigned_upload_url(
        &self,
        filename: &str,
        content_type: &str,
    ) -> Result<aws_sdk_s3::presigning::PresignedRequest, String> {
        let expiration = Duration::from_secs(CONFIG.do_spaces.presigned_expiration);

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
            .key(filename)
            .content_type(content_type)
            .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
            .presigned(presigning_config)
            .await
            .map_err(|e| format!("Failed to generate presigned URL: {}", e))?;

        Ok(presigned_request)
    }
}
