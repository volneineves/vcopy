use crate::config::HistoryConfig;
use crate::storage::{Storage, image_key};
use anyhow::{Context, Result};
use arboard::{Clipboard, Error};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

pub async fn run(storage: Storage, history: HistoryConfig) -> Result<()> {
    let mut clipboard = Clipboard::new().ok();
    let mut last_text = String::new();
    let mut last_image = String::new();
    let mut screenshot_cutoff = SystemTime::now();
    let limit = history.effective_limit();

    if history.clear_on_restart {
        storage.clear()?;
    } else {
        storage.prune(limit)?;
    }

    loop {
        if clipboard.is_none() {
            clipboard = Clipboard::new().ok();
        }

        if let Some(clipboard) = clipboard.as_mut() {
            if let Ok(text) = clipboard.get_text() {
                let text = text.trim().to_string();
                if !text.is_empty() && text != last_text {
                    last_text = text.clone();
                    if let Err(e) = storage.insert_text(&text) {
                        eprintln!("vcopy: {e}");
                    } else if let Err(e) = storage.prune(limit) {
                        eprintln!("vcopy: {e}");
                    }
                }
            }

            match clipboard.get_image() {
                Ok(image) => {
                    let bytes = image.bytes.as_ref();
                    if !bytes.is_empty() {
                        let hash = image_key(image.width, image.height, bytes);
                        if hash != last_image {
                            match storage.insert_image(image.width, image.height, bytes) {
                                Ok(hash) => {
                                    last_image = hash;
                                    if let Err(e) = storage.prune(limit) {
                                        eprintln!("vcopy: {e}");
                                    }
                                }
                                Err(e) => eprintln!("vcopy: {e}"),
                            }
                        }
                    }
                }
                Err(Error::ContentNotAvailable) => {}
                Err(_) => {}
            }
        }

        if let Ok(Some(path)) = latest_screenshot_after(screenshot_cutoff) {
            if let Ok((width, height, bytes)) = read_png_rgba(&path) {
                let hash = image_key(width, height, &bytes);
                if hash != last_image {
                    match storage.insert_image(width, height, &bytes) {
                        Ok(hash) => {
                            last_image = hash;
                            if let Err(e) = storage.prune(limit) {
                                eprintln!("vcopy: {e}");
                            }
                        }
                        Err(e) => eprintln!("vcopy: {e}"),
                    }
                }
            }
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    screenshot_cutoff = modified;
                }
            }
        }

        sleep(Duration::from_millis(500)).await;
    }
}

fn latest_screenshot_after(cutoff: SystemTime) -> Result<Option<PathBuf>> {
    let Some(dir) = dirs::picture_dir().map(|path| path.join("Screenshots")) else {
        return Ok(None);
    };
    if !dir.exists() {
        return Ok(None);
    }

    let mut latest: Option<(SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !is_png(&path) {
            continue;
        }

        let modified = entry.metadata()?.modified()?;
        if modified <= cutoff {
            continue;
        }

        match &latest {
            Some((current, _)) if modified <= *current => {}
            _ => latest = Some((modified, path)),
        }
    }

    Ok(latest.map(|(_, path)| path))
}

fn is_png(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("png"))
        .unwrap_or(false)
}

fn read_png_rgba(path: &Path) -> Result<(usize, usize, Vec<u8>)> {
    let image = image::ImageReader::open(path)
        .with_context(|| format!("failed to open {}", path.display()))?
        .decode()
        .with_context(|| format!("failed to decode {}", path.display()))?
        .to_rgba8();
    let (width, height) = image.dimensions();
    Ok((width as usize, height as usize, image.into_raw()))
}
