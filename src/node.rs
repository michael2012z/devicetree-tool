// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::attribute::Attribute;
use crate::utils::Utils;
use std::rc::Rc;

pub struct Node {
    name: String,
    attributes: Vec<Rc<Attribute>>,
    sub_nodes: Vec<Rc<Node>>,
}

impl Node {
    pub fn new(name: &str) -> Self {
        Node {
            name: String::from(name),
            attributes: Vec::new(),
            sub_nodes: Vec::new(),
        }
    }

    pub fn add_attr(&mut self, attr: Attribute) {
        self.attributes.push(Rc::new(attr));
    }

    pub fn add_sub_node(&mut self, sub_node: Node) {
        self.sub_nodes.push(Rc::new(sub_node));
    }

    pub fn to_dts(&self, indent_level: u32) -> String {
        let mut s = String::new();
        let indents = Utils::indent(indent_level);
        s.push_str(&format!("{indents}"));
        if self.name.len() > 0 {
            s.push_str(&format!("{} ", self.name));
        }
        s.push_str("{\n");
        for attr in self.attributes.iter() {
            s.push_str(&attr.to_dts(indent_level + 1));
            s.push_str("\n");
        }

        for sub_node in self.sub_nodes.iter() {
            s.push_str("\n");
            s.push_str(&sub_node.to_dts(indent_level + 1));
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
        let attr = Attribute::new_u32("attr1", 42);
        let mut node = Node::new("node");
        node.add_attr(attr);
        println!("Node: {}", node.to_dts(0));
    }

    #[test]
    fn test_sub_node() {
        let attr = Attribute::new_u32("attr1", 42);
        let mut node = Node::new("node1");
        node.add_attr(attr);

        let mut sub_node = Node::new("node2");
        sub_node.add_attr(Attribute::new_u32("attr4", 99));

        node.add_sub_node(sub_node);
        println!("Node: {}", node.to_dts(0));
    }
}
