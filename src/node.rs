// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::attribute::Attribute;
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
