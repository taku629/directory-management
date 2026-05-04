//! Image perceptual hashing.
//!
//! Implements dHash (difference hash, 64-bit). Two near-duplicate images
//! produce hashes within a small Hamming distance of each other (≤ 5 by
//! default). It's intentionally simple — no external `img_hash` crate, just
//! the `image` crate we already pull in.

use std::path::Path;

use image::imageops::FilterType;
use image::GenericImageView;

use crate::error::{AppError, AppResult};

/// 9x8 grayscale, row-wise diff → 64 bits.
pub fn dhash(path: &Path) -> AppResult<u64> {
    let img = image::open(path).map_err(|e| AppError::Other(anyhow::anyhow!(e)))?;
    let small = img
        .resize_exact(9, 8, FilterType::Triangle)
        .grayscale()
        .into_luma8();
    let mut bits: u64 = 0;
    for y in 0..8 {
        for x in 0..8 {
            let l = small.get_pixel(x, y).0[0];
            let r = small.get_pixel(x + 1, y).0[0];
            bits = (bits << 1) | (if l > r { 1 } else { 0 });
        }
    }
    Ok(bits)
}

pub fn hex_of(hash: u64) -> String {
    format!("{:016x}", hash)
}

pub fn hash_from_hex(s: &str) -> Option<u64> {
    u64::from_str_radix(s, 16).ok()
}

pub fn hamming(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}
