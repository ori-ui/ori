use gtk4::{
    glib::{object::IsA, subclass::types::ObjectSubclassIsExt as _},
    prelude::WidgetExt as _,
};

use crate::{Context, View};

pub fn min_width<V>(min_width: u32, content: V) -> Size<V> {
    Size::new(content).min_width(min_width)
}

pub fn min_height<V>(min_height: u32, content: V) -> Size<V> {
    Size::new(content).min_height(min_height)
}

pub fn max_width<V>(max_width: u32, content: V) -> Size<V> {
    Size::new(content).max_width(max_width)
}

pub fn max_height<V>(max_height: u32, content: V) -> Size<V> {
    Size::new(content).max_height(max_height)
}

pub fn width<V>(width: u32, content: V) -> Size<V> {
    Size::new(content).width(width)
}

pub fn height<V>(height: u32, content: V) -> Size<V> {
    Size::new(content).height(height)
}

pub fn size<V>(width: u32, height: u32, content: V) -> Size<V> {
    Size::new(content).width(width).height(height)
}

pub struct Size<V> {
    pub content: V,
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
}

impl<V> Size<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            min_width: u32::MIN,
            max_width: u32::MAX,
            min_height: u32::MIN,
            max_height: u32::MAX,
        }
    }

    pub fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = min_height;
        self
    }

    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.min_width = width;
        self.max_width = width;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.min_height = height;
        self.max_height = height;
        self
    }
}

impl<T, V> ori::View<Context, T> for Size<V>
where
    V: View<T>,
{
    type Element = SizeWidget;
    type State = (V::Element, V::State);

    fn build(
        &mut self,
        cx: &mut Context,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let (child, state) = self.content.build(cx, data);

        let element = SizeWidget::new();
        element.set_child(Some(&child));
        element.set_max_width(self.max_width);
        element.set_max_height(self.max_height);

        element.set_size_request(
            self.min_width as i32,
            self.min_height as i32,
        );

        (element, (child, state))
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        (child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.rebuild(child, state, cx, data, &mut old.content);
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        (child, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(child, state, cx, data);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        (child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        self.content.event(child, state, cx, data, event)
    }
}

gtk4::glib::wrapper! {
    pub struct SizeWidget(ObjectSubclass<imp::SizeWidget>)
        @extends
            gtk4::Widget,
        @implements
            gtk4::Accessible,
            gtk4::Buildable,
            gtk4::ConstraintTarget;
}

impl Default for SizeWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl SizeWidget {
    pub fn new() -> Self {
        gtk4::glib::Object::builder().build()
    }

    pub fn set_child(&self, child: Option<&impl IsA<gtk4::Widget>>) {
        let mut current = self.imp().child.borrow_mut();

        if let Some(child) = current.take() {
            child.unparent();
        }

        if let Some(child) = child {
            child.set_parent(self);
            *current = Some(child.as_ref().clone());
        }
    }

    pub fn set_max_width(&self, max_width: u32) {
        self.imp().max_width.set(max_width);
    }

    pub fn set_max_height(&self, max_height: u32) {
        self.imp().max_height.set(max_height);
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk4::{glib, prelude::WidgetExt as _, subclass::prelude::*};

    #[derive(Default)]
    pub struct SizeWidget {
        pub(super) child: RefCell<Option<gtk4::Widget>>,
        pub(super) max_width: Cell<u32>,
        pub(super) max_height: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SizeWidget {
        const NAME: &'static str = "SizeWidget";
        type Type = super::SizeWidget;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for SizeWidget {
        fn dispose(&self) {
            if let Some(ref child) = *self.child.borrow() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for SizeWidget {
        fn measure(
            &self,
            orientation: gtk4::Orientation,
            mut for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let max = match orientation {
                gtk4::Orientation::Horizontal => self.max_width.get() as i32,
                gtk4::Orientation::Vertical => self.max_height.get() as i32,
                _ => i32::MAX,
            };

            if for_size < 0 {
                for_size = max;
            } else {
                for_size = for_size.min(max);
            }

            if let Some(ref child) = *self.child.borrow() {
                let (mut min, mut nat, mut min_baseline, mut nat_baseline) =
                    child.measure(orientation, for_size);

                min = min.min(max);
                nat = nat.min(max);
                min_baseline = min_baseline.min(max);
                nat_baseline = nat_baseline.min(max);

                (min, nat, min_baseline, nat_baseline)
            } else {
                (0, 0, 0, 0)
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            if let Some(ref child) = *self.child.borrow() {
                let width = width.min(self.max_width.get() as i32);
                let height = height.min(self.max_height.get() as i32);

                child.size_allocate(
                    &gtk4::Allocation::new(0, 0, width, height),
                    baseline,
                );
            }
        }
    }
}
