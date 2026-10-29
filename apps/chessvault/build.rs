use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use resvg::{tiny_skia, usvg};

fn main() -> Result<()> {
    println!("cargo::rerun-if-changed=assets/logo.svg");

    let source = fs::read("assets/logo.svg").context("Failed to read window logo")?;
    let tree = usvg::Tree::from_data(&source, &usvg::Options::default()).context("Failed to parse window logo")?;
    let pixmap = render_icon(&tree, 64)?;
    let output = PathBuf::from(env::var_os("OUT_DIR").context("Cargo must set OUT_DIR")?);
    render_icon(&tree, 256)?
        .save_png(output.join("chessvault.png"))
        .context("Failed to write distribution icon")?;

    write_icns(&tree, &output.join("chessvault.icns"))?;

    // tiny-skia stores premultiplied colors; window icons require straight RGBA.
    let rgba: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let pixel = pixel.demultiply();
            [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]
        })
        .collect();
    fs::write(output.join("window-icon.rgba"), rgba).context("Failed to write window icon")
}

// Modern ICNS stores PNG payloads inside big-endian, length-prefixed chunks.
// Include standard and Retina representations; render each from the SVG rather
// than enlarging the 256px Linux distribution icon. This works on every host.
fn write_icns(tree: &usvg::Tree, path: &std::path::Path) -> Result<()> {
    let mut contents = Vec::new();
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
        let png = render_icon(tree, size)?
            .encode_png()
            .context("Failed to encode ICNS image")?;
        let length = u32::try_from(png.len() + 8).context("ICNS image exceeds format size limit")?;
        contents.extend_from_slice(&kind);
        contents.extend_from_slice(&length.to_be_bytes());
        contents.extend_from_slice(&png);
    }
    let length = u32::try_from(contents.len() + 8).context("ICNS file exceeds format size limit")?;
    let mut icon = Vec::with_capacity(contents.len() + 8);
    icon.extend_from_slice(b"icns");
    icon.extend_from_slice(&length.to_be_bytes());
    icon.extend_from_slice(&contents);
    fs::write(path, icon).context("Failed to write macOS distribution icon")
}

fn render_icon(tree: &usvg::Tree, size: u32) -> Result<tiny_skia::Pixmap> {
    let mut pixmap = tiny_skia::Pixmap::new(size, size).context("Failed to allocate window icon")?;
    let transform =
        tiny_skia::Transform::from_scale(size as f32 / tree.size().width(), size as f32 / tree.size().height());
    resvg::render(tree, transform, &mut pixmap.as_mut());
    Ok(pixmap)
}
