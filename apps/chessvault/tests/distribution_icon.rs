//! Validate the actual ICNS emitted by the build script on any host.

use std::io::Cursor;

#[test]
fn macos_icon_contains_standard_and_retina_pngs() {
    let icon = include_bytes!(concat!(env!("OUT_DIR"), "/chessvault.icns"));
    assert_eq!(&icon[..4], b"icns");
    let length = u32::from_be_bytes(icon[4..8].try_into().unwrap()) as usize;
    assert_eq!(length, icon.len());

    let mut remaining = &icon[8..];
    for (kind, size) in [
        (*b"icp4", 16),
        (*b"icp5", 32),
        (*b"icp6", 64),
        (*b"ic07", 128),
        (*b"ic08", 256),
        (*b"ic09", 512),
        (*b"ic10", 1024),
        (*b"ic11", 32),
        (*b"ic12", 64),
        (*b"ic13", 256),
        (*b"ic14", 512),
    ] {
        assert_eq!(&remaining[..4], &kind);
        let length = u32::from_be_bytes(remaining[4..8].try_into().unwrap()) as usize;
        assert!(length > 8 && length <= remaining.len());
        let mut reader = png::Decoder::new(Cursor::new(&remaining[8..length]))
            .read_info()
            .unwrap();
        assert_eq!(reader.info().width, size);
        assert_eq!(reader.info().height, size);
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut pixels).unwrap();
        assert!(pixels.iter().any(|byte| *byte != 0));
        remaining = &remaining[length..];
    }
    assert!(remaining.is_empty(), "unexpected trailing ICNS data");
}
