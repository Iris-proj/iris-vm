use std::{collections::HashMap, rc::Rc};
use crate::vm::function::Function;
use crate::vm::value::Value;

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub type_id: usize,
    pub superclass: Option<Rc<Class>>,
    pub methods: Vec<Rc<Function>>,
    pub properties: HashMap<String, usize>,
}

impl Class {
    pub fn new(name: String, type_id: usize, superclass: Option<Rc<Class>>) -> Self {
        Self {
            name,
            type_id,
            superclass,
            methods: Vec::new(),
            properties: HashMap::new(),
        }
    }

    pub fn add_method(&mut self, key: usize, method: Rc<Function>) {
        self.methods.insert(key, method);
    }

    pub fn find_method(&self, key: usize) -> Option<Rc<Function>> {
        if let Some(method) = self.methods.get(key) {
            Some(method.clone())
        } else if let Some(ref super_cls) = self.superclass {
            super_cls.find_method(key)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Instance {
    pub class: Rc<Class>,
    pub fields: Vec<Value>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            fields: Vec::new(),
        }
    }

    pub fn get_method(&self, key: usize) -> Option<Rc<Function>> {
        self.class.find_method(key)
    }

    pub fn get_field(&self, key: usize) -> Option<&Value> {
        self.fields.get(key)
    }

    pub fn set_field(&mut self, key: usize, value: Value) {
        self.fields.insert(key, value);
    }
}
