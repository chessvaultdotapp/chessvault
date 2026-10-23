use std::{io::ErrorKind, path::Path};

use anyhow::{Context, Result};
use iced::window;
use tracing::{info, warn};

pub fn load() -> Result<window::Icon> {
    match platform_dirs::user_share_dir(None) {
        Ok(base) => {
            let path = base.join("icons/hicolor/256x256/apps/chessvault.png");
            match load_installed(&path) {
                Ok(Some(icon)) => return Ok(icon),
                Ok(None) => info!(path = %path.display(), "Installed icon unavailable; using embedded icon"),
                Err(error) => {
                    warn!(path = %path.display(), error = ?error, "Failed to load installed icon; using embedded icon")
                }
            }
        }
        Err(error) => warn!(error = ?error, "Cannot resolve installed icon directory; using embedded icon"),
    }
    load_embedded()
}

fn load_installed(path: &Path) -> Result<Option<window::Icon>> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("Failed to read installed icon"),
    };
    decode_png(&bytes).map(Some)
}

fn decode_png(bytes: &[u8]) -> Result<window::Icon> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().context("Failed to decode installed icon")?;
    let mut pixels = vec![0; reader.output_buffer_size().context("Installed icon is too large")?];
    let output = reader
        .next_frame(&mut pixels)
        .context("Failed to decode installed icon pixels")?;
    let pixels = &pixels[..output.buffer_size()];
    let rgba = match output.color_type {
        png::ColorType::Rgba => pixels.to_vec(),
        png::ColorType::Rgb => pixels
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::Grayscale => pixels.iter().flat_map(|&p| [p, p, p, 255]).collect(),
        png::ColorType::GrayscaleAlpha => pixels
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Indexed => anyhow::bail!("Installed icon palette was not expanded"),
    };
    window::icon::from_rgba(rgba, output.width, output.height).context("Failed to create installed window icon")
}

fn load_embedded() -> Result<window::Icon> {
    let rgba = include_bytes!(concat!(env!("OUT_DIR"), "/window-icon.rgba"));
    window::icon::from_rgba(rgba.to_vec(), 64, 64).context("Failed to create window icon")
}

#[cfg(test)]
mod tests {
    #[test]
    fn installed_png_produces_window_icon() {
        let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/chessvault.png"));
        let (rgba, size) = super::decode_png(bytes).unwrap().into_raw();
        assert_eq!((size.width, size.height), (256, 256));
        assert_eq!(rgba.len(), 256 * 256 * 4);
    }

    #[test]
    fn invalid_installed_png_is_rejected() {
        assert!(super::decode_png(b"not a PNG").is_err());
    }

    #[test]
    fn missing_installed_icon_is_not_an_error() {
        let path = std::env::temp_dir()
            .join(format!("chessvault-missing-icon-{}", std::process::id()))
            .join("icon.png");
        assert!(super::load_installed(&path).unwrap().is_none());
    }

    #[test]
    fn embedded_logo_produces_window_icon() {
        let (rgba, size) = super::load_embedded().unwrap().into_raw();
        assert_eq!((size.width, size.height), (64, 64));
        assert_eq!(rgba.len(), 64 * 64 * 4);
        let (pixels, _) = rgba.as_chunks::<4>();
        assert!(pixels.iter().any(|pixel| pixel[3] == 255));
        assert!(pixels.iter().any(|pixel| pixel[3] == 0));
    }
}
