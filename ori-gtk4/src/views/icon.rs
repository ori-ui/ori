use gtk4::{
    gdk_pixbuf::prelude::PixbufLoaderExt as _, prelude::WidgetExt as _,
    subclass::prelude::ObjectSubclassIsExt as _,
};

use crate::Context;

pub fn icon(svg: impl Into<String>) -> Icon {
    Icon::new(svg)
}

pub struct Icon {
    svg: String,
}

impl Icon {
    pub fn new(svg: impl Into<String>) -> Self {
        Self { svg: svg.into() }
    }
}

impl ori::ViewMarker for Icon {}
impl<T> ori::View<Context, T> for Icon {
    type Element = IconWidget;
    type State = ();

    fn build(&mut self, _cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let element = IconWidget::new();

        element.set_svg(&self.svg);

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
        if self.svg != old.svg {
            element.set_svg(&self.svg);
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

gtk4::glib::wrapper! {
    pub struct IconWidget(ObjectSubclass<imp::IconWidget>)
        @extends
            gtk4::Widget,
        @implements
            gtk4::Accessible,
            gtk4::Buildable,
            gtk4::ConstraintTarget;
}

impl Default for IconWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl IconWidget {
    pub fn new() -> Self {
        gtk4::glib::Object::new()
    }

    pub fn set_svg(&self, data: &str) {
        *self.imp().texture.borrow_mut() = Self::load_svg(data);
        self.queue_draw();
    }

    fn load_svg(data: &str) -> Option<gtk4::gdk::Texture> {
        let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
        let _ = loader.write(data.as_bytes());
        let _ = loader.close();
        let pixbuf = loader.pixbuf()?;
        Some(gtk4::gdk::Texture::for_pixbuf(&pixbuf))
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk4::{
        gdk, glib,
        graphene::{Matrix, Rect, Vec4},
        prelude::*,
        subclass::prelude::*,
    };

    #[derive(Default)]
    pub struct IconWidget {
        pub(super) texture: RefCell<Option<gdk::Texture>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconWidget {
        const NAME: &'static str = "Icon";
        type Type = super::IconWidget;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("icon");
        }
    }

    impl ObjectImpl for IconWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_overflow(gtk4::Overflow::Hidden);
            self.obj().set_focusable(false);
        }
    }

    impl WidgetImpl for IconWidget {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let Some(ref texture) = *self.texture.borrow() else {
                return;
            };

            let color = self.obj().style_context().color();
            self.obj().style_context().scale();

            let alloc = self.obj().allocation();

            let aspect = texture.width() as f32 / texture.height() as f32;
            let rel_width = alloc.width() as f32 / aspect;
            let rel_height = alloc.height() as f32;

            let height = rel_height.min(rel_width);
            let width = height * aspect;

            let width_overflow = alloc.width() as f32 - width;
            let height_overflow = alloc.height() as f32 - height;

            let bounds = Rect::new(
                width_overflow / 2.0,
                height_overflow / 2.0,
                width,
                height,
            );

            let r = color.red();
            let g = color.green();
            let b = color.blue();
            let a = color.alpha();

            let matrix = Matrix::from_float([
                0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, //
                r, g, b, a, //
            ]);

            let offset = Vec4::zero();

            snapshot.push_color_matrix(&matrix, &offset);
            snapshot.append_texture(texture, &bounds);
            snapshot.pop();
        }
    }
}
