use std::{env, fs, path::Path};

const ROBOTO_BLACK: &[u8] = include_bytes!("font/Roboto-Black.ttf");
const ROBOTO_BLACK_ITALIC: &[u8] = include_bytes!("font/Roboto-BlackItalic.ttf");
const ROBOTO_BOLD: &[u8] = include_bytes!("font/Roboto-Bold.ttf");
const ROBOTO_BOLD_ITALIC: &[u8] = include_bytes!("font/Roboto-BoldItalic.ttf");
const ROBOTO_ITALIC: &[u8] = include_bytes!("font/Roboto-Italic.ttf");
const ROBOTO_LIGHT: &[u8] = include_bytes!("font/Roboto-Light.ttf");
const ROBOTO_LIGHT_ITALIC: &[u8] = include_bytes!("font/Roboto-LightItalic.ttf");
const ROBOTO_MEDIUM: &[u8] = include_bytes!("font/Roboto-Medium.ttf");
const ROBOTO_MEDIUM_ITALIC: &[u8] = include_bytes!("font/Roboto-MediumItalic.ttf");
const ROBOTO_REGULAR: &[u8] = include_bytes!("font/Roboto-Regular.ttf");
const ROBOTO_THIN: &[u8] = include_bytes!("font/Roboto-Thin.ttf");
const ROBOTO_THIN_ITALIC: &[u8] = include_bytes!("font/Roboto-ThinItalic.ttf");

const ROBOTO_MONO_BOLD: &[u8] = include_bytes!("font/RobotoMono-Bold.ttf");
const ROBOTO_MONO_BOLD_ITALIC: &[u8] = include_bytes!("font/RobotoMono-BoldItalic.ttf");
const ROBOTO_MONO_ITALIC: &[u8] = include_bytes!("font/RobotoMono-Italic.ttf");
const ROBOTO_MONO_LIGHT: &[u8] = include_bytes!("font/RobotoMono-Light.ttf");
const ROBOTO_MONO_LIGHT_ITALIC: &[u8] = include_bytes!("font/RobotoMono-LightItalic.ttf");
const ROBOTO_MONO_MEDIUM: &[u8] = include_bytes!("font/RobotoMono-Medium.ttf");
const ROBOTO_MONO_MEDIUM_ITALIC: &[u8] = include_bytes!("font/RobotoMono-MediumItalic.ttf");
const ROBOTO_MONO_REGULAR: &[u8] = include_bytes!("font/RobotoMono-Regular.ttf");
const ROBOTO_MONO_THIN: &[u8] = include_bytes!("font/RobotoMono-Thin.ttf");
const ROBOTO_MONO_THIN_ITALIC: &[u8] = include_bytes!("font/RobotoMono-ThinItalic.ttf");

const FONTS: &[&[u8]] = &[
    ROBOTO_BLACK,
    ROBOTO_BLACK_ITALIC,
    ROBOTO_BOLD,
    ROBOTO_BOLD_ITALIC,
    ROBOTO_ITALIC,
    ROBOTO_LIGHT,
    ROBOTO_LIGHT_ITALIC,
    ROBOTO_MEDIUM,
    ROBOTO_MEDIUM_ITALIC,
    ROBOTO_REGULAR,
    ROBOTO_THIN,
    ROBOTO_THIN_ITALIC,
    ROBOTO_MONO_BOLD,
    ROBOTO_MONO_BOLD_ITALIC,
    ROBOTO_MONO_ITALIC,
    ROBOTO_MONO_LIGHT,
    ROBOTO_MONO_LIGHT_ITALIC,
    ROBOTO_MONO_MEDIUM,
    ROBOTO_MONO_MEDIUM_ITALIC,
    ROBOTO_MONO_REGULAR,
    ROBOTO_MONO_THIN,
    ROBOTO_MONO_THIN_ITALIC,
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = Vec::new();

    // write the number of fonts
    input.extend_from_slice(&u32::to_le_bytes(FONTS.len() as u32));

    for font in FONTS {
        // write the length of the font
        input.extend_from_slice(&u32::to_le_bytes(font.len() as u32));

        // write the font data
        input.extend_from_slice(font);
    }

    let compressed = miniz_oxide::deflate::compress_to_vec(&input, 10);
    let out = env::var("OUT_DIR")?;

    // write the output to the file
    fs::write(Path::new(&out).join("fonts.bin"), &compressed)?;

    Ok(())
}
