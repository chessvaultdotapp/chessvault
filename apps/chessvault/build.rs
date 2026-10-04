use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use resvg::{tiny_skia, usvg};

fn main() -> Result<()> {
    println!("cargo::rerun-if-changed=assets/logo.svg");

    let source = fs::read("assets/logo.svg").context("Failed to read window logo")?;
    let tree = usvg::Tree::from_data(&source, &usvg::Options::default()).context("Failed to parse window logo")?;
    let size = 64;
    let mut pixmap = tiny_skia::Pixmap::new(size, size).context("Failed to allocate window icon")?;
    let transform =
        tiny_skia::Transform::from_scale(size as f32 / tree.size().width(), size as f32 / tree.size().height());
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // tiny-skia stores premultiplied colors; window icons require straight RGBA.
    let rgba: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let pixel = pixel.demultiply();
            [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]
        })
        .collect();
    let output = PathBuf::from(env::var_os("OUT_DIR").context("Cargo must set OUT_DIR")?);
    fs::write(output.join("window-icon.rgba"), rgba).context("Failed to write window icon")
}
