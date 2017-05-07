use std::io;
use std::fs;
use std::path::Path;

use common::config::ValidatedPlaylist;
use daemon::processor::playlist::Image;

pub fn scan_playlist(config: &ValidatedPlaylist) -> Vec<Image> {
    let mut files: Vec<_> = Vec::new();

    for file in &config.files {
        if check_file(file) {
            files.push(Image {
                path: file.clone(),
                usable: true
            });
        }
    }

    for dir in &config.directories {
        if let Err(e) = scan_directory(dir, &mut files) {
            warn!("Error reading directory {}: {}", dir.display(), e);
        }
    }

    files
}

pub struct RescanResult {
    pub added: Vec<Image>,
    pub removed: Vec<Image>,
}

pub enum SingleRescanResult {
    Added(Image),
    Removed(Image),
    NotChanged,
}

pub fn rescan_playlist(config: &ValidatedPlaylist, existing_images: &[Image]) -> RescanResult {
    let mut total_added = Vec::new();
    let mut total_removed = Vec::new();

    for file in &config.files {
        match rescan_file(file, existing_images) {
            SingleRescanResult::Added(image) => total_added.push(image),
            SingleRescanResult::Removed(image) => total_removed.push(image),
            SingleRescanResult::NotChanged => {}
        }
    }

    for dir in &config.directories {
        match rescan_directory(dir, existing_images) {
            Ok(RescanResult { added, removed, }) => {
                total_added.extend(added);
                total_removed.extend(removed);
            }
            Err(e) => warn!("Error rescanning directory '{}': {}", dir.display(), e)
        }
    }

    RescanResult {
        added: total_added,
        removed: total_removed,
    }
}

fn rescan_file(file: &Path, existing_images: &[Image]) -> SingleRescanResult {
    if check_file(file) {
        if existing_images.iter().any(|img| img.path == file) {
            SingleRescanResult::NotChanged
        } else {
            info!("Configured file '{}' is available again", file.display());
            SingleRescanResult::Added(Image {
                path: file.into(),
                usable: true,
            })
        }

    } else {
        if let Some(image) = existing_images.iter().find(|img| img.path == file) {
            warn!("Configured file '{}' is no longer available", file.display());
            SingleRescanResult::Removed(image.clone())
        } else {
            SingleRescanResult::NotChanged
        }
    }
}

fn rescan_directory(dir: &Path, existing_images: &[Image]) -> io::Result<RescanResult> {
    let mut added = Vec::new();
    let mut removed: Vec<_> = existing_images.iter().collect();

    for entry in fs::read_dir(dir)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Error reading directory entry inside '{}': {}", dir.display(), e);
                continue;
            }
        };

        let path = entry.path();

        if let Some(idx) = removed.iter().position(|img| img.path == path) {
            removed.remove(idx);
        }

        if !existing_images.iter().any(|img| img.path == path) && check_file(&path) {
            added.push(Image {
                path: path,
                usable: true,
            })
        }
    }

    if added.len() > 0 {
        if removed.len() > 0 {
            info!("Found {} new files and lost {} existing files in directory '{}'", added.len(), removed.len(), dir.display());
        } else {
            info!("Found {} new files in directory '{}'", added.len(), dir.display());
        }
    } else if removed.len() > 0 {
        info!("Lost {} existing files in directory '{}'", removed.len(), dir.display());
    }

    Ok(RescanResult {
        added: added,
        removed: removed.into_iter().cloned().collect(),
    })
}

fn scan_directory(dir: &Path, images: &mut Vec<Image>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Error reading directory entry inside '{}': {}", dir.display(), e);
                continue;
            }
        };

        let path = entry.path();
        if check_file(&path) {
            images.push(Image {
                path: path,
                usable: true,
            });
        }
    }
    Ok(())
}

pub fn check_file(file: &Path) -> bool {
    if check_file_format(file) {
        match check_file_access(file) {
            Ok(r) => r,
            Err(e) => {
                warn!("Error accessing file {}: {}", file.display(), e);
                false
            }
        }
    } else {
        false
    }
}

fn check_file_format(file: &Path) -> bool {
    if let Some(ext) = file.extension() {
        if let Some(ext) = ext.to_str() {
            match &*ext.to_lowercase() {
                "jpg" | "jpeg" | "png" | "gif" => return true,
                _ => {}
            }
        }
    }
    false
}

fn check_file_access(file: &Path) -> io::Result<bool> {
    fs::metadata(file).map(|m| m.is_file())
}
