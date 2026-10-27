use std::io::Write;

use anyhow::{Context, Result};
use iced::{Size, window};
use tracing::{error, info};

mod app;
mod board;
mod fs;
mod game_status;
mod history;
mod icon;
mod pgn;

use app::{Message, State};
mod logs;
mod serialization;

fn main() -> Result<()> {
    let logs = logs::Logs::init()?;
    info!("ChessVault started");

    let mut window_settings = window::Settings {
        icon: match icon::load() {
            Ok(icon) => Some(icon),
            Err(err) => {
                error!(error = ?err, "Failed to load window icon");
                None
            }
        },
        ..window::Settings::default()
    };
    #[cfg(target_os = "linux")]
    {
        window_settings.platform_specific.application_id = "chessvault".into();
    }
    match WindowRecreateInfo::load() {
        Ok(Some(info)) => window_settings.size = Size::new(info.width, info.height),
        Ok(None) => {
            info!("No saved window size found; using default size");
        }
        Err(err) => error!(error = ?err, "Failed to restore window size; using default size"),
    }

    iced::application(move || State::boot(logs.clone()), State::update, State::view)
        .title("ChessVault")
        .window(window_settings)
        .exit_on_close_request(false)
        .subscription(State::subscription)
        .run()
        .context("Failed to run ChessVault")
        .inspect_err(|err| error!(error = ?err, "Application exited with an error"))
}

#[derive(Debug)]
struct WindowRecreateInfo {
    width: f32,
    height: f32,
}

impl WindowRecreateInfo {
    fn load() -> Result<Option<Self>> {
        let path = fs::window_recreate_info_filepath().context("Failed to resolve window recreate info filepath")?;

        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(err).with_context(|| format!("Failed to read {}", path.display()));
            }
        };

        Self::deserialize(&bytes)
            .with_context(|| format!("Failed to load window recreate info from {}", path.display()))
            .map(Some)
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(24);
        bytes.extend_from_slice(&serialization::header_for(serialization::Format::WindowRecreateInfo));
        bytes.extend_from_slice(&self.width.to_le_bytes());
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self> {
        anyhow::ensure!(bytes.len() == 24, "Saved window info must be exactly 24 bytes");
        anyhow::ensure!(
            bytes[..16] == serialization::header_for(serialization::Format::WindowRecreateInfo),
            "Invalid or unsupported saved window info header"
        );
        let info = Self {
            width: f32::from_le_bytes(bytes[16..20].try_into()?),
            height: f32::from_le_bytes(bytes[20..24].try_into()?),
        };
        anyhow::ensure!(
            info.width.is_finite() && info.height.is_finite() && info.width > 0.0 && info.height > 0.0,
            "Saved window dimensions must be finite and positive"
        );
        Ok(info)
    }

    fn save(&self) -> Result<()> {
        let path = fs::window_recreate_info_filepath().context("Failed to resolve window recreate info filepath")?;

        let bytes = self.serialize();

        let mut file =
            std::fs::File::create(&path).with_context(|| format!("Failed to create or truncate {}", path.display()))?;

        file.write_all(&bytes)
            .with_context(|| format!("Failed to write window recreate info to {}", path.display()))?;

        Ok(())
    }
}

impl From<Size<f32>> for WindowRecreateInfo {
    fn from(value: Size<f32>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

#[cfg(test)]
mod window_recreate_tests {
    use super::*;

    #[test]
    fn saved_dimensions_round_trip() {
        let bytes = WindowRecreateInfo::from(Size::new(820.5, 620.0)).serialize();
        let info = WindowRecreateInfo::deserialize(&bytes).unwrap();
        assert_eq!(info.width, 820.5);
        assert_eq!(info.height, 620.0);
    }

    #[test]
    fn saved_dimensions_exact_bytes() {
        assert_eq!(
            WindowRecreateInfo::from(Size::new(820.5, 620.0)).serialize(),
            [
                b'C', b'H', b'S', b'V', 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 32, 77, 68, 0, 0, 27, 68,
            ]
        );
    }

    #[test]
    fn invalid_headers_are_rejected() {
        let bytes = WindowRecreateInfo::from(Size::new(820.5, 620.0)).serialize();
        for index in 0..16 {
            let mut invalid = bytes.clone();
            invalid[index] ^= 1;
            assert!(WindowRecreateInfo::deserialize(&invalid).is_err(), "byte {index}");
        }
    }

    #[test]
    fn invalid_lengths_and_legacy_json_are_rejected() {
        let bytes = WindowRecreateInfo::from(Size::new(820.5, 620.0)).serialize();
        for length in 0..24 {
            assert!(WindowRecreateInfo::deserialize(&bytes[..length]).is_err());
        }
        let mut extended = bytes.to_vec();
        extended.push(0);
        assert!(WindowRecreateInfo::deserialize(&extended).is_err());
        assert!(WindowRecreateInfo::deserialize(br#"{"width":820,"height":620}"#).is_err());
    }

    #[test]
    fn invalid_dimensions_are_rejected() {
        for invalid in [0.0, -0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for size in [Size::new(invalid, 620.0), Size::new(820.0, invalid)] {
                let bytes = WindowRecreateInfo::from(size).serialize();
                assert!(WindowRecreateInfo::deserialize(&bytes).is_err(), "{size:?}");
            }
        }
    }
}
