use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use iced::window;
use tracing::{info, warn};

pub fn load() -> Result<window::Icon> {
    let mut bases = Vec::new();
    match platform_dirs::user_share_dir(None) {
        Ok(base) => bases.push(base),
        Err(error) => warn!(error = ?error, "Cannot resolve user icon directory; continuing icon search"),
    }
    match platform_dirs::system_share_dirs(None) {
        Ok(system) => bases.extend(system),
        Err(error) => warn!(error = ?error, "Cannot resolve system icon directories; continuing icon search"),
    }
    if let Some(icon) = load_from_dirs(bases, load_installed) {
        return Ok(icon);
    }
    info!("No installed icon available; using embedded icon");
    load_embedded()
}

fn load_from_dirs(
    bases: impl IntoIterator<Item = PathBuf>,
    mut load: impl FnMut(&Path) -> Result<Option<window::Icon>>,
) -> Option<window::Icon> {
    for base in bases {
        let path = base.join("icons/hicolor/256x256/apps/chessvault.png");
        match load(&path) {
            Ok(Some(icon)) => return Some(icon),
            Ok(None) => {}
            Err(error) => {
                warn!(path = %path.display(), error = ?error, "Failed to load installed icon; continuing icon search")
            }
        }
    }
    None
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
    fn installed_search_preserves_order_and_stops_at_first_icon() {
        let mut visited = Vec::new();
        let icon = super::load_from_dirs(
            ["user", "system-first", "system-second", "unused"].map(std::path::PathBuf::from),
            |path| {
                visited.push(path.to_path_buf());
                match visited.len() {
                    1 => Ok(None),
                    2 => anyhow::bail!("unreadable or invalid icon"),
                    3 => super::load_embedded().map(Some),
                    _ => panic!("search continued after finding an icon"),
                }
            },
        );
        assert!(icon.is_some());
        assert_eq!(
            visited,
            ["user", "system-first", "system-second"]
                .map(|base| std::path::Path::new(base).join("icons/hicolor/256x256/apps/chessvault.png"))
        );
    }

    #[test]
    fn exhausted_or_empty_search_returns_none() {
        assert!(super::load_from_dirs(Vec::new(), |_| panic!("empty search")).is_none());
        assert!(super::load_from_dirs([std::path::PathBuf::from("missing")], |_| Ok(None)).is_none());
    }

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
