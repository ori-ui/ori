use ori_core::{UiBuilder, View};

use crate::Error;

pub struct App<T> {
    pub(crate) builder: UiBuilder<T>,
    pub(crate) data: T,
}

impl<T: 'static> App<T> {
    pub fn new<V>(mut builder: impl FnMut(&mut T) -> V + 'static, data: T) -> Self
    where
        V: View<T> + 'static,
        V::State: 'static,
    {
        Self {
            builder: Box::new(move |data| Box::new(builder(data))),
            data,
        }
    }

    pub fn try_run(self) -> Result<(), Error> {
        crate::run::run(self)
    }

    pub fn run(self) {
        if let Err(err) = self.try_run() {
            panic!("{}", err);
        }
    }
}
