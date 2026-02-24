use super::FileInfo;

use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;

pub fn extract(path: &Path) -> FileInfo {
    match ffprobe::ffprobe(path) {
        Ok(meta) => {
            let tags = meta.format.tags.as_ref();
            let model = tags
                .and_then(|t| {
                    t.extra
                        .get("model")
                        .or_else(|| t.extra.get("com.apple.quicktime.model"))
                        .or_else(|| t.extra.get("AndroidModel"))
                        .or_else(|| t.extra.get("make"))
                        .and_then(|v| v.as_str())
                })
                .map(|s| s.to_string())
                .unwrap_or_else(|| "UnknownModel".to_string());

            let date = tags
                .and_then(|t| t.creation_time.as_ref())
                .and_then(|dt_str| {
                    DateTime::parse_from_rfc3339(dt_str)
                        .map(|dt| dt.with_timezone(&Utc).naive_utc())
                        .ok()
                });

            FileInfo { date, model }
        }
        Err(e) => {
            eprintln!(
                "No FFProbe metadata in {} ({}), using defaults",
                path.display(),
                e
            );
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
