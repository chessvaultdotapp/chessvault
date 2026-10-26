//! Shared binary record headers for desktop persistence.
//!
//! The 16-byte header is `CHSV`, followed by little-endian `u32` major version,
//! minor version, and format identifier. The current version is 0.1. Consumers
//! validate the exact header, not a compatible version range; changing any header
//! field therefore requires an explicit reader/migration policy.
//!
//! Window records append little-endian `f32` width and height, for 24 bytes total.
//! The window reader rejects legacy JSON and unsupported headers/lengths; startup
//! logs the failure and uses defaults. No automatic migration is performed.

const BINARY_FORMAT_MAGIC_BYTES: [u8; 4] = *b"CHSV";
const VERSION_MAJOR_BYTES: [u8; 4] = 0_u32.to_le_bytes();
const VERSION_MINOR_BYTES: [u8; 4] = 1_u32.to_le_bytes();

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Format {
    /// Window dimensions; payload is width then height as little-endian `f32`.
    WindowRecreateInfo = 0,
}

impl Format {
    pub const fn to_le_bytes(self) -> [u8; 4] {
        (self as u32).to_le_bytes()
    }
}

/// Returns the exact current header expected by a reader of this format.
pub const fn header_for(format: Format) -> [u8; 16] {
    let format_bytes = format.to_le_bytes();
    let mut header = [0; 16];
    let mut i = 0;

    while i < 4 {
        header[i] = BINARY_FORMAT_MAGIC_BYTES[i];
        header[4 + i] = VERSION_MAJOR_BYTES[i];
        header[8 + i] = VERSION_MINOR_BYTES[i];
        header[12 + i] = format_bytes[i];
        i += 1;
    }

    header
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_recreate_header_bytes() {
        const HEADER: [u8; 16] = header_for(Format::WindowRecreateInfo);

        assert_eq!(
            HEADER,
            [
                b'C', b'H', b'S', b'V', 0, 0, 0, 0, // Major version.
                1, 0, 0, 0, // Minor version.
                0, 0, 0, 0, // Format.
            ]
        );
    }
}
