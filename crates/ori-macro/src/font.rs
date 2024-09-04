use std::{
    env, fs,
    hash::{DefaultHasher, Hasher},
    io,
    path::{Path, PathBuf},
};

use quote::quote;
use syn::LitStr;

use crate::find_core;

fn load_dir(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut fonts = Vec::new();

    for entry in path.read_dir()? {
        let entry = entry?;
        let input = load_font(&entry.path())?;
        fonts.extend(input);
    }

    Ok(fonts)
}

fn load_file(path: &Path) -> io::Result<Vec<PathBuf>> {
    Ok(vec![path.to_owned()])
}

fn load_font(path: &Path) -> io::Result<Vec<PathBuf>> {
    if path.is_dir() {
        load_dir(path)
    } else {
        load_file(path)
    }
}

pub fn include_font(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    let input: LitStr = syn::parse(input)?;

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);
    let path = manifest_dir.join(input.value());

    let fonts = match load_font(&path) {
        Ok(style) => style,
        Err(err) => manyhow::bail!(input, "io error: {}", err),
    };

    let mut compressed = Vec::new();

    compressed.extend_from_slice(&u32::to_le_bytes(fonts.len() as u32));

    for font in fonts {
        let data = fs::read(&font).unwrap();

        compressed.extend_from_slice(&u32::to_le_bytes(data.len() as u32));
        compressed.extend_from_slice(&data);
    }

    let compressed = miniz_oxide::deflate::compress_to_vec(&compressed, 10);

    let mut hasher = DefaultHasher::new();

    hasher.write(&compressed);

    let file_name = format!("font-{}.bin", hasher.finish());

    let out_dir = env!("OUT_DIR");
    let bin_path = Path::new(&out_dir).join(&file_name);

    if !bin_path.exists() {
        fs::write(&bin_path, &compressed).unwrap();
    }

    let bin_path_str = bin_path.to_string_lossy();

    let ori_core = find_core();

    let expanded = quote! {
        #ori_core::text::FontSource::Bundle(
            ::std::borrow::Cow::Borrowed(::std::include_bytes!(#bin_path_str))
        )
    };

    Ok(expanded.into())
}
