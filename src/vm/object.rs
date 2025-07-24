use std::{collections::HashMap, rc::Rc};
use crate::vm::function::Function;
use crate::vm::value::Value;

pub struct Class {
    pub name: String,
    pub type_id: usize,
    pub superclass: Option<Rc<Class>>,
    pub methods: HashMap<String, Rc<Function>>,
    pub properties: Vec<String>,
}

impl Class {
    pub fn new(name: String, type_id: usize, superclass: Option<Rc<Class>>) -> Self {
        Self {
            name,
            type_id,
            superclass,
            methods: HashMap::new(),
            properties: Vec::new(),
        }
    }

    pub fn add_method(&mut self, name: String, method: Rc<Function>) {
        self.methods.insert(name, method);
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<Function>> {
        if let Some(method) = self.methods.get(name) {
            Some(method.clone())
        } else if let Some(ref super_cls) = self.superclass {
            super_cls.find_method(name)
        } else {
            None
        }
    }
}

pub struct Instance {
    pub class: Rc<Class>,
    pub fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get_method(&self, name: &str) -> Option<Rc<Function>> {
        self.class.find_method(name)
    }

    pub fn get_field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    pub fn set_field(&mut self, name: String, value: Value) {
        self.fields.insert(name, value);
    }
}
