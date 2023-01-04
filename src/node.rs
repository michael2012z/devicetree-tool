// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::attribute::Attribute;
use crate::dts_generator::DtsGenerator;
use std::rc::Rc;

pub struct Node {
    pub name: String,
    pub attributes: Vec<Rc<Attribute>>,
    pub sub_nodes: Vec<Rc<Node>>,
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
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = DtsGenerator::generate_node(self, 0);
        writeln!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::Attribute;

    #[test]
    fn test_node_empty() {
        let node = Node::new("node");
        assert_eq!(node.attributes.len(), 0);
        assert_eq!(node.sub_nodes.len(), 0);
    }

    #[test]
    fn test_node_sub_nodes() {
        let mut node = Node::new("node");
        node.add_sub_node(Node::new("sub_node_0"));
        node.add_sub_node(Node::new("sub_node_1"));
        node.add_sub_node(Node::new("sub_node_2"));

        let mut sub_node_3 = Node::new("sub_node_3");
        sub_node_3.add_sub_node(Node::new("sub_node_30"));
        sub_node_3.add_sub_node(Node::new("sub_node_31"));

        assert_eq!(sub_node_3.sub_nodes.len(), 2);

        node.add_sub_node(sub_node_3);
        assert_eq!(node.sub_nodes.len(), 4);
    }

    #[test]
    fn test_node_attributes() {
        let mut node = Node::new("node");
        node.add_attr(Attribute::new_empty("attr0"));
        node.add_attr(Attribute::new_u32("attr1", 42));
        assert_eq!(node.attributes.len(), 2);
    }

    #[test]
    fn test_attribute_print() {
        let mut node = Node::new("node");
        node.add_attr(Attribute::new_u32("attr", 42));
        let mut sub_node = Node::new("node");
        sub_node.add_attr(Attribute::new_u32("attr", 12));
        node.add_sub_node(sub_node);

        let printing = format!("{}", node);
        assert_eq!(&printing, "node {\n\tattr = <0x0 0x0 0x0 0x2a>;\n\n\tnode {\n\t\tattr = <0x0 0x0 0x0 0xc>;\n\t};\n};\n");
    }
}
