// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::attribute::InternalAttribute;
use std::sync::{Arc, Mutex};

pub struct Node {
    attributes: Vec<Arc<Mutex<dyn InternalAttribute>>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            attributes: Vec::new(),
        }
    }

    pub fn add_attr(&mut self, attr: Arc<Mutex<dyn InternalAttribute>>) {
        self.attributes.push(attr);
    }

    pub fn to_dts(&self) -> String {
        let mut s = String::new();
        for attr in self.attributes.iter() {
            s.push_str(&attr.lock().unwrap().to_dts());
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::Attribute;

    #[test]
    fn test_node() {
        let attr = Arc::new(Mutex::new(Attribute::new("name", 42u32)));
        let mut node = Node::new();
        node.add_attr(attr);
        node.add_attr(Arc::new(Mutex::new(Attribute::new("name", 12.3456f32))));
        println!("Node: {}", node.to_dts());
    }
}
