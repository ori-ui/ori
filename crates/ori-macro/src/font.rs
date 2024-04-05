use std::{env, io, path::Path};

use quote::quote;
use syn::LitStr;

use crate::find_core;

fn load_dir(path: &Path) -> io::Result<Vec<String>> {
    let mut fonts = Vec::new();

    for entry in path.read_dir()? {
        let entry = entry?;
        let input = load_font(&entry.path())?;
        fonts.extend(input);
    }

    Ok(fonts)
}

fn load_file(path: &Path) -> io::Result<Vec<String>> {
    Ok(vec![path.to_string_lossy().into_owned()])
}

fn load_font(path: &Path) -> io::Result<Vec<String>> {
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

    let ori_core = find_core();

    let expanded = quote! {
        #ori_core::text::FontSource::Set(::std::vec![#(
            #ori_core::text::FontSource::Data(::std::include_bytes!(#fonts).to_vec())
        ),*])
    };

    Ok(expanded.into())
}
