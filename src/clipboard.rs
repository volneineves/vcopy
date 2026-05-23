use crate::storage::{ClipItem, ClipKind, Storage};
use anyhow::{Context, Result, anyhow};
use arboard::{Clipboard, ImageData};
use std::borrow::Cow;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_item_to_clipboard(storage: &Storage, item: &ClipItem) -> Result<()> {
    match item.kind {
        ClipKind::Text => copy_text_to_clipboard(&item.content),
        ClipKind::Image => copy_image_to_clipboard(storage, item),
    }
}

fn copy_image_to_clipboard(storage: &Storage, item: &ClipItem) -> Result<()> {
    let bytes = storage.read_image(item)?;
    let width = item.image_width.context("image width missing")?;
    let height = item.image_height.context("image height missing")?;

    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(()) = copy_image_with_wl_copy(width, height, &bytes) {
            return Ok(());
        }
    }

    let image = ImageData {
        width,
        height,
        bytes: Cow::Owned(bytes),
    };
    Clipboard::new()?.set_image(image)?;
    Ok(())
}

fn copy_image_with_wl_copy(width: usize, height: usize, bytes: &[u8]) -> Result<()> {
    let png = encode_png(width, height, bytes)?;
    let mut child = Command::new("wl-copy")
        .args(["--type", "image/png"])
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to start wl-copy")?;

    let stdin = child
        .stdin
        .as_mut()
        .context("failed to open wl-copy stdin")?;
    stdin
        .write_all(&png)
        .context("failed to write image to wl-copy")?;

    let status = child.wait().context("failed to wait for wl-copy")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("wl-copy exited with {status}"))
    }
}

fn encode_png(width: usize, height: usize, bytes: &[u8]) -> Result<Vec<u8>> {
    let mut png = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png, width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(bytes)?;
    }
    Ok(png)
}

fn copy_text_to_clipboard(content: &str) -> Result<()> {
    match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(content.to_owned())) {
        Ok(()) => Ok(()),
        Err(arboard_err) if std::env::var_os("WAYLAND_DISPLAY").is_some() => {
            copy_text_with_wl_copy(content).with_context(|| format!("arboard failed: {arboard_err}"))
        }
        Err(arboard_err) => Err(anyhow!(arboard_err)),
    }
}

fn copy_text_with_wl_copy(content: &str) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to start wl-copy")?;

    let stdin = child
        .stdin
        .as_mut()
        .context("failed to open wl-copy stdin")?;
    stdin
        .write_all(content.as_bytes())
        .context("failed to write clipboard content to wl-copy")?;

    let status = child.wait().context("failed to wait for wl-copy")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("wl-copy exited with {status}"))
    }
}
