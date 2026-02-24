use chrono::{DateTime, NaiveDateTime, Utc};
use exif::{Exif, In, Tag};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use twox_hash::XxHash3_64;
use walkdir::WalkDir;

mod move_file;
use crate::move_file::move_file;

struct Config {
    input_dir: PathBuf,
    output_dir: PathBuf,
    copy_only: bool,
}

struct FileInfo {
    date: Option<NaiveDateTime>,
    model: String,
}

fn extract_file_info(exif: &Exif) -> FileInfo {
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

fn generate_filename(info: &FileInfo, file_bytes: &[u8]) -> String {
    let timestamp = info
        .date
        .map(|dt| dt.format("%Y%m%d_%H%M%S").to_string())
        .unwrap_or_else(|| "UnknownDate".to_string());

    let hash_val = XxHash3_64::oneshot(file_bytes);
    let short_hash = format!("{:016x}", hash_val)[..8].to_string();

    format!("{}_{}_{}", timestamp, info.model, short_hash)
}

fn build_output_path(output_root: &Path, info: &FileInfo, filename: &str) -> PathBuf {
    match info.date {
        Some(dt) => output_root
            .join(dt.format("%Y").to_string())
            .join(dt.format("%m").to_string())
            .join(dt.format("%d").to_string())
            .join(filename),
        None => output_root.join("UnknownDate").join(filename),
    }
}

fn process_with_info(
    config: &Config,
    path: &Path,
    info: &FileInfo,
    file_bytes: &[u8],
    extension: &str,
) {
    let new_file_stem = generate_filename(info, file_bytes);
    let filename = format!("{}.{}", new_file_stem, extension);
    let new_path = build_output_path(&config.output_dir, info, &filename);

    if new_path.exists() {
        eprintln!(
            "Skipping {}: target {} already exists",
            path.display(),
            new_path.display()
        );
        return;
    }

    if let Some(parent) = new_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create directory {}: {}", parent.display(), e);
            return;
        }
    }

    let result = if config.copy_only {
        fs::copy(path, &new_path).map(|_| ())
    } else {
        move_file(path, &new_path)
    };

    let action_verb = if config.copy_only { "Copy" } else { "Move" };

    match result {
        Ok(_) => {
            println!(
                "{} successful: {} -> {}",
                action_verb,
                path.display(),
                new_path.display()
            )
        }
        Err(e) => {
            eprintln!(
                "Failed to {} {} -> {}: {}",
                action_verb.to_lowercase(),
                path.display(),
                new_path.display(),
                e
            )
        }
    }
}

fn process_file(config: &Config, path: &Path) {
    let mut buffer = Vec::new();

    {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open {}: {}", path.display(), e);
                return;
            }
        };

        if let Err(e) = file.read_to_end(&mut buffer) {
            eprintln!("Failed to read {}: {}", path.display(), e);
            return;
        }
    }

    let extension = match path.extension().and_then(OsStr::to_str) {
        Some(ext) => ext,
        None => {
            eprintln!("Skipping {}: no extension", path.display());
            return;
        }
    };

    let info = match exif::Reader::new().read_from_container(&mut Cursor::new(&buffer)) {
        Ok(exif) => extract_file_info(&exif),
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
    };

    process_with_info(config, path, &info, &buffer, extension);
}

fn main() {
    let input_raw = shellexpand::tilde("~/Photolibrary/lysi_samples").to_string();
    let output_raw = shellexpand::tilde("~/Pictures/Organized").to_string();
    let config = Config {
        input_dir: PathBuf::from(input_raw),
        output_dir: PathBuf::from(output_raw),
        copy_only: true,
    };

    if !config.input_dir.exists() {
        eprintln!(
            "Input directory does not exist: {}",
            config.input_dir.display()
        );
        return;
    }

    for entry in WalkDir::new(&config.input_dir) {
        match entry {
            Ok(entry) if entry.path().is_file() => process_file(&config, entry.path()),
            Err(e) => eprintln!("Error walking directory: {}", e),
            _ => {}
        }
    }
}
