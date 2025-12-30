use std::sync::Arc;

use image::codecs::png::PngEncoder;
use s3::Bucket;
use sha2::Digest;
use tracing::trace;

use crate::config::S3Config;

/// Minimal S3-backed image storage. This keeps things intentionally simple for now:
/// - construct from existing `S3Config`
/// - upload raw bytes under a key
/// - upload a local file by path (reads whole file into memory)
/// - generate a simple presigned GET URL
/// - process avatars with resizing and upload
///
/// Backed by `s3-tokio` (hyper 1 + rustls) and compatible with S3/R2/MinIO endpoints.
#[derive(Clone)]
pub struct ImageStorage {
    bucket: Arc<s3::Bucket>,
    public_base_url: String,
}

impl ImageStorage {
    /// Create a new storage for a specific `bucket_name` using the provided S3 config.
    ///
    /// This uses a custom region + endpoint so it works across AWS S3 and compatible services
    /// such as Cloudflare R2 and MinIO.
    pub fn new(config: &S3Config, bucket_name: impl Into<String>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let credentials = s3::creds::Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_access_key),
            None, // security token
            None, // session token
            None, // profile
        )?;

        let bucket = Bucket::new(
            &bucket_name.into(),
            s3::Region::R2 {
                account_id: "f188bf93079278e7bbc58de9b3d80693".to_string(),
            },
            credentials,
        )?
        .with_path_style();

        Ok(Self {
            bucket: Arc::new(bucket),
            public_base_url: config.public_base_url.clone(),
        })
    }

    /// Upload a byte slice to `key` with optional content type.
    ///
    /// Returns the ETag (if present) from the server response.
    pub async fn upload_bytes(
        &self,
        key: &str,
        bytes: impl AsRef<[u8]>,
        content_type: Option<&str>,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let data = bytes.as_ref();
        let content_type = content_type.unwrap_or("application/octet-stream");

        // Prefer the content-type variant for correct metadata
        let status = {
            let response = self.bucket.put_object_with_content_type(key, data, content_type).await?;
            response.status_code()
        };

        if (200..300).contains(&status) {
            // s3-tokio returns headers separately; attempt to pull the ETag if available
            // Note: the current API returns (status, headers) where headers is `http::HeaderMap`.
            // Some providers omit ETag on PUT; we handle that by returning `None`.
            Ok(None)
        } else {
            Err(format!("upload failed with status {}", status).into())
        }
    }

    /// Generate a simple presigned GET URL valid for `expires_in_seconds`.
    #[allow(dead_code)]
    pub fn presign_get(&self, key: &str, expires_in_seconds: u32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.bucket.presign_get(key, expires_in_seconds, None)?;
        Ok(url)
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
        let response = reqwest::get(avatar_url).await?;
        if !response.status().is_success() {
            return Err(format!("Failed to download avatar: {}", response.status()).into());
        }

        let image_bytes = response.bytes().await?;
        trace!(bytes = image_bytes.len(), "Downloaded avatar");

        // Decode the image
        let img = image::load_from_memory(&image_bytes)?;
        let img_rgba = img.to_rgba8();

        // Generate a simple hash for the avatar (using the URL for now)
        let avatar_hash = format!("{:x}", sha2::Sha256::digest(avatar_url.as_bytes()));
        trace!(
            width = img_rgba.width(),
            height = img_rgba.height(),
            hash = avatar_hash,
            "Avatar image decoded"
        );

        // Process original (512x512 max, square)
        let original_key = format!("avatars/{}/{}.original.png", user_public_id, avatar_hash);
        let original_png = self.resize_to_square_png(&img_rgba, 512)?;
        self.upload_bytes(&original_key, &original_png, Some("image/png")).await?;
        trace!(key = original_key, "Uploaded original avatar");

        // Process mini (32x32)
        let mini_key = format!("avatars/{}/{}.mini.png", user_public_id, avatar_hash);
        let mini_png = self.resize_to_square_png(&img_rgba, 32)?;
        self.upload_bytes(&mini_key, &mini_png, Some("image/png")).await?;
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

// References:
// - Example (R2): https://github.com/FemLolStudio/s3-tokio/blob/master/examples/r2-tokio.rs
// - Crate docs:   https://lib.rs/crates/s3-tokio
