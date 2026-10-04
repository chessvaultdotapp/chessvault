use anyhow::{Context, Result};
use iced::window;

pub fn load() -> Result<window::Icon> {
    let rgba = include_bytes!(concat!(env!("OUT_DIR"), "/window-icon.rgba"));
    window::icon::from_rgba(rgba.to_vec(), 64, 64).context("Failed to create window icon")
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_logo_produces_window_icon() {
        let (rgba, size) = super::load().unwrap().into_raw();
        assert_eq!((size.width, size.height), (64, 64));
        assert_eq!(rgba.len(), 64 * 64 * 4);
        let (pixels, _) = rgba.as_chunks::<4>();
        assert!(pixels.iter().any(|pixel| pixel[3] == 255));
        assert!(pixels.iter().any(|pixel| pixel[3] == 0));
    }
}
