use std::{env, fs, path::PathBuf};

use resvg::{tiny_skia, usvg};

fn main() {
    println!("cargo::rerun-if-changed=assets/logo.svg");

    let source = fs::read("assets/logo.svg").expect("Failed to read window logo");
    let tree = usvg::Tree::from_data(&source, &usvg::Options::default()).expect("Failed to parse window logo");
    let size = 64;
    let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("Failed to allocate window icon");
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
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    fs::write(output.join("window-icon.rgba"), rgba).expect("Failed to write window icon");
}
