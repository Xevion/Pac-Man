use std::time::Duration;

use image::codecs::png::PngEncoder;
use rusty_s3::actions::{PutObject, S3Action as _};
use rusty_s3::{Bucket, Credentials, UrlStyle};
use sha2::Digest;
use tracing::trace;

use crate::config::S3Config;

/// How long a presigned upload URL stays valid; uploads start immediately after signing.
const PRESIGN_TTL: Duration = Duration::from_secs(60);

/// Minimal S3-compatible image storage (Cloudflare R2).
///
/// Requests are presigned with `rusty-s3` and sent with the shared `reqwest` client, so this
/// adds no second HTTP or TLS stack.
#[derive(Clone)]
pub struct ImageStorage {
    bucket: Bucket,
    credentials: Credentials,
    http: reqwest::Client,
    public_base_url: String,
}

impl ImageStorage {
    /// Create a new storage for a specific `bucket_name` in the configured R2 account.
    pub fn new(config: &S3Config, bucket_name: impl Into<String>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint = format!("https://{}.r2.cloudflarestorage.com", config.account_id).parse()?;
        Self::with_endpoint(endpoint, "auto", config, bucket_name)
    }

    /// Create a storage against an arbitrary S3-compatible endpoint, such as a local MinIO.
    pub fn with_endpoint(
        endpoint: reqwest::Url,
        region: &str,
        config: &S3Config,
        bucket_name: impl Into<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let bucket = Bucket::new(endpoint, UrlStyle::Path, bucket_name.into(), region.to_owned())?;

        Ok(Self {
            bucket,
            credentials: Credentials::new(&config.access_key, &config.secret_access_key),
            http: crate::http::client_builder().build()?,
            public_base_url: config.public_base_url.clone(),
        })
    }

    /// Upload a byte slice to `key` with optional content type.
    pub async fn upload_bytes(
        &self,
        key: &str,
        bytes: impl Into<reqwest::Body>,
        content_type: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = PutObject::new(&self.bucket, Some(&self.credentials), key).sign(PRESIGN_TTL);
        let response = self
            .http
            .put(url)
            .header(
                reqwest::header::CONTENT_TYPE,
                content_type.unwrap_or("application/octet-stream"),
            )
            .body(bytes)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(format!("upload of {key} failed with status {status}: {body}").into())
        }
    }

    /// Process and upload an avatar from a URL.
    ///
    /// Downloads the image, resizes it to 512x512 (original) and 32x32 (mini),
    /// then uploads both versions to S3. Returns the public URLs for both images.
    pub async fn process_avatar(
        &self,
        user_public_id: &str,
        avatar_url: &str,
    ) -> Result<AvatarUrls, Box<dyn std::error::Error + Send + Sync>> {
        // Download the avatar image
        let response = self.http.get(avatar_url).send().await?;
        if !response.status().is_success() {
            return Err(format!("Failed to download avatar: {}", response.status()).into());
        }

        let image_bytes = response.bytes().await?;
        trace!(bytes = image_bytes.len(), "Downloaded avatar");

        // Decode the image
        let img = image::load_from_memory(&image_bytes)?;
        let img_rgba = img.to_rgba8();

        // Generate a simple hash for the avatar (using the URL for now)
        let avatar_hash = sha2::Sha256::digest(avatar_url.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        trace!(
            width = img_rgba.width(),
            height = img_rgba.height(),
            hash = avatar_hash,
            "Avatar image decoded"
        );

        // Process original (512x512 max, square)
        let original_key = format!("avatars/{}/{}.original.png", user_public_id, avatar_hash);
        let original_png = self.resize_to_square_png(&img_rgba, 512)?;
        self.upload_bytes(&original_key, original_png, Some("image/png")).await?;
        trace!(key = original_key, "Uploaded original avatar");

        // Process mini (32x32)
        let mini_key = format!("avatars/{}/{}.mini.png", user_public_id, avatar_hash);
        let mini_png = self.resize_to_square_png(&img_rgba, 32)?;
        self.upload_bytes(&mini_key, mini_png, Some("image/png")).await?;
        trace!(key = mini_key, "Uploaded mini avatar");

        Ok(AvatarUrls {
            original_url: format!("{}/{}", self.public_base_url, original_key),
            mini_url: format!("{}/{}", self.public_base_url, mini_key),
        })
    }

    /// Resize an RGBA image to a square of the specified size, maintaining aspect ratio.
    fn resize_to_square_png(
        &self,
        img: &image::RgbaImage,
        target_size: u32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let (width, height) = img.dimensions();

        // Calculate dimensions for square crop (center crop)
        let size = width.min(height);
        let start_x = (width - size) / 2;
        let start_y = (height - size) / 2;

        // Crop to square
        let cropped = image::imageops::crop_imm(img, start_x, start_y, size, size).to_image();

        // Resize to target size
        let resized = image::imageops::resize(&cropped, target_size, target_size, image::imageops::FilterType::Lanczos3);

        // Encode as PNG
        let mut bytes: Vec<u8> = Vec::new();
        let cursor = std::io::Cursor::new(&mut bytes);

        // Write the resized image to the cursor
        resized.write_with_encoder(PngEncoder::new(cursor))?;

        Ok(bytes)
    }
}

/// URLs for processed avatar images
#[derive(Debug, Clone)]
pub struct AvatarUrls {
    pub original_url: String,
    pub mini_url: String,
}

impl ImageStorage {
    /// Create a new storage using the bucket from `S3Config`.
    pub fn from_config(config: &S3Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::new(config, &config.bucket_name)
    }
}
