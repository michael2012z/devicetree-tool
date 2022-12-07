// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::element::Element;
use std::sync::{Arc, Mutex};

pub struct Node {
    name: String,
    attributes: Vec<Arc<Mutex<dyn Element>>>,
    sub_nodes: Vec<Arc<Mutex<Node>>>,
}

impl Node {
    pub fn new(name: &str) -> Self {
        Node {
            name: String::from(name),
            attributes: Vec::new(),
            sub_nodes: Vec::new(),
        }
    }

    pub fn add_attr(&mut self, attr: Arc<Mutex<dyn Element>>) {
        self.attributes.push(attr);
    }

    pub fn add_sub_node(&mut self, sub_node: Node) {
        self.sub_nodes.push(Arc::new(Mutex::new(sub_node)));
    }
}

impl Element for Node {
    fn to_dts(&self, indent_level: u32) -> String {
        let mut indents = String::new();
        for i in 0..indent_level {
            indents.push('\t')
        }

        let mut s = String::new();
        s.push_str(&format!("{indents}"));
        if self.name.len() > 0 {
            s.push_str(&format!("{} ", self.name));
        }
        s.push_str("{\n");
        for attr in self.attributes.iter() {
            s.push_str(&attr.lock().unwrap().to_dts(indent_level + 1));
            s.push_str("\n");
        }

        if self.sub_nodes.len() > 0 {
            s.push_str("\n");
        }

        for sub_node in self.sub_nodes.iter() {
            s.push_str(&sub_node.lock().unwrap().to_dts(indent_level + 1));
            s.push_str("\n");
        }
        s.push_str(&format!("{indents}}};"));
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::Attribute;

    #[test]
    fn test_simple_node() {
        let attr = Arc::new(Mutex::new(Attribute::new("attr1", 42u32)));
        let mut node = Node::new("node");
        node.add_attr(attr);
        println!("Node: {}", node.to_dts(0));
    }

    #[test]
    fn test_sub_node() {
        let attr = Arc::new(Mutex::new(Attribute::new("attr1", 42u32)));
        let mut node = Node::new("node1");
        node.add_attr(attr);

        let mut sub_node = Node::new("node2");
        sub_node.add_attr(Arc::new(Mutex::new(Attribute::new("attr4", 99u32))));

        node.add_sub_node(sub_node);
        println!("Node: {}", node.to_dts(0));
    }
}
