use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use json::JsonValue;

const ICONS_JSON_PATH: &str = "font/icons.json";

#[allow(dead_code)]
#[derive(Debug)]
enum Error {
    Io(io::Error),
    Json(json::Error),
    Var(env::VarError),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<json::Error> for Error {
    fn from(error: json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<env::VarError> for Error {
    fn from(error: env::VarError) -> Self {
        Self::Var(error)
    }
}

#[derive(Debug)]
enum Font {
    Regular,
    Solid,
    Brand,
}

impl Font {
    fn from_value(value: &JsonValue) -> Option<Self> {
        match value.as_str()? {
            "regular" => Some(Self::Regular),
            "solid" => Some(Self::Solid),
            "brands" => Some(Self::Brand),
            _ => None,
        }
    }
}

fn kebab_to_pascal(kebab: &str) -> String {
    let mut pascal = String::new();

    for word in kebab.split('-') {
        let mut chars = word.chars();

        if let Some(first) = chars.next() {
            pascal.extend(first.to_uppercase());

            for c in chars {
                pascal.push(c);
            }
        }
    }

    pascal
}

#[derive(Debug)]
struct Icon {
    name: String,
    label: String,
    unicode: String,
    fonts: Vec<Font>,
}

impl Icon {
    fn new((name, value): (&str, &JsonValue)) -> Option<Self> {
        let label = String::from(value["label"].as_str()?);
        let unicode = String::from(value["unicode"].as_str()?);
        let fonts = (value["free"].members())
            .filter_map(Font::from_value)
            .collect();

        Some(Self {
            name: String::from(name),
            label,
            unicode,
            fonts,
        })
    }

    fn ident(&self) -> String {
        let mut pascal = kebab_to_pascal(&self.name);

        if pascal.starts_with(char::is_numeric) {
            pascal = format!("Num{}", pascal);
        }

        pascal
    }
}

fn generate(f: &mut impl Write, icons: &[Icon]) -> io::Result<()> {
    /* generate the Icon enum */

    writeln!(f, "/// An icon code.")?;
    writeln!(f, "#[non_exhaustive]")?;
    writeln!(f, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]")?;
    writeln!(f, "pub enum IconCode {{")?;

    for icon in icons {
        writeln!(
            f,
            "/// The [`{}`](https://fontawesome.com/icons/{}) icon.",
            icon.label, icon.name,
        )?;
        writeln!(f, "{},", icon.ident())?;
    }

    writeln!(f, "}}")?;

    /* generate the Icon impl */

    writeln!(f, "impl IconCode {{")?;

    /* generate ALL_ICONS */

    writeln!(f, "/// All the icons.")?;
    writeln!(f, "pub const ALL_ICONS: &'static [Self] = &[")?;

    for icon in icons {
        writeln!(f, "Self::{},", icon.ident())?;
    }

    writeln!(f, "];")?;

    /* generate the Icon::code_point method */

    writeln!(f, "/// The code point of the icon.")?;
    writeln!(f, "pub fn code_point(self) -> char {{")?;
    writeln!(f, "match self {{")?;

    for icon in icons {
        writeln!(f, "Self::{} => '\\u{{{}}}',", icon.ident(), icon.unicode)?;
    }

    writeln!(f, "}}")?;
    writeln!(f, "}}")?;

    /* generate the Icon::as_str method */

    writeln!(f, "/// The code point of the icon as a `&str`.")?;
    writeln!(f, "pub fn as_str(self) -> &'static str {{")?;
    writeln!(f, "match self {{")?;

    for icon in icons {
        writeln!(f, "Self::{} => \"\\u{{{}}}\",", icon.ident(), icon.unicode)?;
    }

    writeln!(f, "}}")?;
    writeln!(f, "}}")?;

    /* generate the Icon::label method */

    writeln!(f, "/// The label of the icon.")?;
    writeln!(f, "pub fn label(self) -> &'static str {{")?;
    writeln!(f, "match self {{")?;

    for icon in icons {
        writeln!(f, "Self::{} => \"{}\",", icon.ident(), icon.label)?;
    }

    writeln!(f, "}}")?;
    writeln!(f, "}}")?;

    /* generate the Icon::font method */

    writeln!(f, "/// The font of the icon.")?;
    writeln!(f, "pub fn fonts(self) -> &'static [IconFont] {{")?;
    writeln!(f, "match self {{")?;

    for icon in icons {
        writeln!(f, "Self::{} => &[", icon.ident())?;

        for font in &icon.fonts {
            writeln!(f, "IconFont::{:?},", font)?;
        }

        writeln!(f, "],")?;
    }

    writeln!(f, "}}")?;
    writeln!(f, "}}")?;

    /* generate the Icon::from_name method */

    writeln!(f, "/// Get the icon from the name.")?;
    writeln!(f, "pub fn from_name(name: &str) -> Option<Self> {{")?;
    writeln!(f, "match name {{")?;

    for icon in icons {
        writeln!(f, "\"{}\" => Some(Self::{}),", icon.name, icon.ident())?;
    }

    writeln!(f, "_ => None,")?;
    writeln!(f, "}}")?;
    writeln!(f, "}}")?;

    /* end the Icon impl */

    writeln!(f, "}}")?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let out_dir = env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);

    let icons_json = fs::read_to_string(ICONS_JSON_PATH)?;
    let icons = json::parse(&icons_json)?;

    let icons: Vec<_> = icons.entries().flat_map(Icon::new).collect();

    let mut output = File::create(out_dir.join("icons.rs"))?;

    generate(&mut output, &icons)?;

    Ok(())
}
