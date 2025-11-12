use std::path::{Path, PathBuf};

use gtk4::gdk_pixbuf::prelude::PixbufLoaderExt;

use crate::Context;

pub fn picture(source: impl Into<ImageSource>) -> Picture {
    Picture::new(source)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageSource {
    File(PathBuf),

    Svg(String),

    #[cfg(feature = "image")]
    Data(image::RgbaImage),
}

impl From<&str> for ImageSource {
    fn from(path: &str) -> Self {
        ImageSource::File(path.into())
    }
}

impl From<String> for ImageSource {
    fn from(path: String) -> Self {
        ImageSource::File(path.into())
    }
}

impl From<&Path> for ImageSource {
    fn from(path: &Path) -> Self {
        ImageSource::File(path.into())
    }
}

impl From<PathBuf> for ImageSource {
    fn from(path: PathBuf) -> Self {
        ImageSource::File(path)
    }
}

#[cfg(feature = "image")]
impl From<image::RgbaImage> for ImageSource {
    fn from(data: image::RgbaImage) -> Self {
        ImageSource::Data(data)
    }
}

pub struct Picture {
    source: ImageSource,
}

impl Picture {
    pub fn new(source: impl Into<ImageSource>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

fn load_from_source(element: &gtk4::Picture, source: &ImageSource) {
    match source {
        ImageSource::File(path) => {
            element.set_filename(Some(path));
        }

        ImageSource::Svg(data) => {
            let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
            let _ = loader.write(data.as_bytes());
            let _ = loader.close();
            let pixbuf = loader.pixbuf().unwrap();

            element.set_pixbuf(Some(&pixbuf));
        }

        #[cfg(feature = "image")]
        ImageSource::Data(data) => {
            use std::io::Read;

            let texture = gtk4::gdk::MemoryTexture::new(
                data.width() as i32,
                data.height() as i32,
                gtk4::gdk::MemoryFormat::R8g8b8a8,
                &gtk4::glib::Bytes::from_owned(
                    data.bytes().collect::<Result<Vec<_>, _>>().unwrap(),
                ),
                data.width() as usize * 4,
            );

            element.set_paintable(Some(&texture));
        }
    }
}

impl ori::ViewMarker for Picture {}
impl<T> ori::View<Context, T> for Picture {
    type Element = gtk4::Picture;
    type State = ();

    fn build(&mut self, _cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let element = gtk4::Picture::new();

        load_from_source(&element, &self.source);

        element.set_can_shrink(true);

        (element, ())
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        if self.source != old.source {
            load_from_source(element, &self.source);
        }
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        _state: Self::State,
        _cx: &mut Context,
        _data: &mut T,
    ) {
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _event: &mut ori::Event,
    ) -> ori::Action {
        ori::Action::new()
    }
}
