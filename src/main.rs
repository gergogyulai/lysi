use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod metadata;
mod move_file;

use crate::move_file::move_file;
use metadata::{FileInfo, build_output_path, generate_filename};

struct Config {
    input_dir: PathBuf,
    output_dir: PathBuf,
    copy_only: bool,
}

fn process_with_info(
    config: &Config,
    path: &Path,
    info: &FileInfo,
    file_bytes: &[u8],
    extension: &str,
) {
    let filename = format!("{}.{}", generate_filename(info, file_bytes), extension);
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

    let action = if config.copy_only { "Copy" } else { "Move" };
    match result {
        Ok(_) => println!(
            "{} successful: {} -> {}",
            action,
            path.display(),
            new_path.display()
        ),
        Err(e) => eprintln!(
            "Failed to {} {} -> {}: {}",
            action.to_lowercase(),
            path.display(),
            new_path.display(),
            e
        ),
    }
}

fn process_file(config: &Config, path: &Path) {
    let extension = match path.extension().and_then(OsStr::to_str) {
        Some(ext) => ext,
        None => {
            eprintln!("Skipping {}: no extension", path.display());
            return;
        }
    };

    let mut buffer = Vec::new();
    if let Err(e) = File::open(path).and_then(|mut f| f.read_to_end(&mut buffer)) {
        eprintln!("Failed to read {}: {}", path.display(), e);
        return;
    }

    let info = match metadata::extract(extension, &buffer, path) {
        Some(info) => info,
        None => {
            eprintln!("Skipping {}: unsupported file type", path.display());
            return;
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
