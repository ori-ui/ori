use std::{any::Any, collections::HashMap, sync::Arc};

#[derive(Clone, Default)]
pub struct Style {
    classes: HashMap<String, Attributes>,
}

impl Style {}

#[derive(Clone, Default)]
pub struct Attributes {
    attributes: Vec<Attribute>,
}

impl Attributes {
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    pub fn find(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.name() == name)
    }

    pub fn set<T: Any>(&mut self, name: &str, value: T) {
        self.attributes.push(Attribute::new(name, value));
    }

    pub fn get<T: Any>(&self, name: &str) -> Option<&T> {
        self.find(name)?.get()
    }
}

#[derive(Clone)]
pub struct Attribute {
    name: String,
    value: Arc<dyn Any>,
}

impl Attribute {
    pub fn new<T: Any>(name: &str, value: T) -> Self {
        Self {
            name: name.to_string(),
            value: Arc::new(value),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }
}
