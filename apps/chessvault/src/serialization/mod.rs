const BINARY_FORMAT_MAGIC_BYTES: [u8; 4] = *b"CHSV";
const VERSION_MAJOR_BYTES: [u8; 4] = 0_u32.to_le_bytes();
const VERSION_MINOR_BYTES: [u8; 4] = 1_u32.to_le_bytes();

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Format {
    WindowRecreateInfo = 0,
}

impl Format {
    pub const fn to_le_bytes(self) -> [u8; 4] {
        (self as u32).to_le_bytes()
    }
}

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
