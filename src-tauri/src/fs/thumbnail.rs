//! Thumbnail generation. Phase 1 supports image thumbs via the `image` crate;
//! Phase 2/3 will add video (ffmpeg sidecar), PDF (pdfium), and code preview.

use std::path::Path;

use image::imageops::FilterType;

use crate::error::{AppError, AppResult};

const THUMB_SIZE: u32 = 256;

/// Generate a JPEG thumbnail for an image and return raw bytes.
pub fn thumbnail_image(path: &Path) -> AppResult<Vec<u8>> {
    let img = image::open(path).map_err(|e| AppError::Other(anyhow::anyhow!(e)))?;
    let thumb = img.resize(THUMB_SIZE, THUMB_SIZE, FilterType::Triangle);
    let mut out = std::io::Cursor::new(Vec::new());
    thumb
        .write_to(&mut out, image::ImageFormat::Jpeg)
        .map_err(|e| AppError::Other(anyhow::anyhow!(e)))?;
    Ok(out.into_inner())
}
