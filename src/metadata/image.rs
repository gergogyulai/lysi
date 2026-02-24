use super::FileInfo;

use chrono::{DateTime, NaiveDateTime, Utc};
use exif::{In, Tag};
use std::fs;
use std::io::Cursor;
use std::path::Path;

pub fn extract(bytes: &[u8], path: &Path) -> FileInfo {
    match exif::Reader::new().read_from_container(&mut Cursor::new(bytes)) {
        Ok(exif) => {
            let date = exif
                .get_field(Tag::DateTimeOriginal, In::PRIMARY)
                .map(|f| f.display_value().to_string())
                .and_then(|s| NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok());

            let model = exif
                .get_field(Tag::Model, In::PRIMARY)
                .map(|f| f.display_value().to_string().replace('"', ""))
                .unwrap_or_else(|| "UnknownModel".to_string());

            FileInfo { date, model }
        }
        Err(e) => {
            eprintln!("No EXIF data in {} ({}), using defaults", path.display(), e);
            let date = fs::metadata(path)
                .ok()
                .and_then(|m| m.created().ok())
                .map(|t| DateTime::<Utc>::from(t).naive_utc());

            FileInfo {
                date,
                model: "UnknownModel".to_string(),
            }
        }
    }
}
