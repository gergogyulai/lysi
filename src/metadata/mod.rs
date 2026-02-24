pub mod image;
pub mod video;

use chrono::NaiveDateTime;
use std::path::{Path, PathBuf};
use twox_hash::XxHash3_64;

pub struct FileInfo {
    pub date: Option<NaiveDateTime>,
    pub model: String,
}

pub fn extract(extension: &str, bytes: &[u8], path: &Path) -> Option<FileInfo> {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "avif" | "tiff" | "tif" | "bmp"
        | "cr2" | "nef" | "arw" | "dng" | "orf" | "srw" => Some(image::extract(bytes, path)),
        "mp4" | "m4v" | "mov" | "qt" | "mkv" | "avi" | "wmv" | "flv" | "webm" | "mts" | "m2ts"
        | "mxf" | "ogv" | "3gp" => Some(video::extract(path)),
        _ => None,
    }
}

pub fn generate_filename(info: &FileInfo, file_bytes: &[u8]) -> String {
    let timestamp = info
        .date
        .map(|dt| dt.format("%Y%m%d_%H%M%S").to_string())
        .unwrap_or_else(|| "UnknownDate".to_string());

    let short_hash = format!("{:016x}", XxHash3_64::oneshot(file_bytes))[..8].to_string();

    format!("{}_{}_{}", timestamp, info.model, short_hash)
}

pub fn build_output_path(output_root: &Path, info: &FileInfo, filename: &str) -> PathBuf {
    match info.date {
        Some(dt) => output_root
            .join(dt.format("%Y").to_string())
            .join(dt.format("%m").to_string())
            .join(dt.format("%d").to_string())
            .join(filename),
        None => output_root.join("UnknownDate").join(filename),
    }
}
