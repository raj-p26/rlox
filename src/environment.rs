#![allow(unused)]

use std::collections::HashMap;

use crate::{token::Token, Lox};

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, String>,
    pub enclosing: Option<Box<Environment>>
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Environment) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn define(&mut self, name: String, value: String) {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: Token) -> Option<String> {
        if self.values.contains_key(&name.lexeme) {
            let val = self.values.get(&name.lexeme).unwrap().clone();
            return Some(val);
        }

        if self.enclosing.is_some() {
            let mut enclosing = self.enclosing.as_mut().unwrap();
            let val = enclosing.get(name.clone());
            if let Some(val) = val {
                return Some(val);
            }
        }

        Lox::error(name.line, format!("Undefined Variable '{}'.", name.lexeme));
        None
    }

    pub fn assign(&mut self, name: Token, value: String) -> Option<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Some(());
        }

        if let Some(enclosed_env) = self.enclosing.as_mut() {
            enclosed_env.assign(name.clone(), value);
            return Some(());
        }

        Lox::report(
            name.line,
            format!("at '{}'", name.lexeme),
            format!("Undefined Variable '{}'.", name.lexeme)
            );
        None
    }
}
