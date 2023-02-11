// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dts_generator::DtsGenerator;
use crate::property::Property;
use std::sync::{Arc, Mutex};

pub struct Node {
    pub name: String,
    pub label: Option<String>,
    pub properties: Vec<Arc<Mutex<Property>>>,
    pub sub_nodes: Vec<Arc<Mutex<Node>>>,
}

impl Node {
    pub fn new(name: &str) -> Self {
        Node {
            name: String::from(name),
            label: None,
            properties: Vec::new(),
            sub_nodes: Vec::new(),
        }
    }

    pub fn new_label(name: &str, label: &str) -> Self {
        Node {
            name: String::from(name),
            label: Some(String::from(label)),
            properties: Vec::new(),
            sub_nodes: Vec::new(),
        }
    }

    pub fn add_property(&mut self, prop: Property) {
        self.properties.push(Arc::new(Mutex::new(prop)));
    }

    pub fn add_sub_node(&mut self, sub_node: Node) {
        self.sub_nodes.push(Arc::new(Mutex::new(sub_node)));
    }

    pub fn find_property(&self, name: &str) -> Option<Arc<Mutex<Property>>> {
        for prop in &self.properties {
            if name == prop.lock().unwrap().name {
                return Some(prop.clone());
            }
        }
        None
    }

    pub fn find_subnode_by_name(&self, name: &str) -> Option<Arc<Mutex<Node>>> {
        for sub_node in &self.sub_nodes {
            if sub_node.lock().unwrap().name == name {
                return Some(sub_node.clone());
            }
        }
        None
    }

    pub fn find_subnode_with_label(&self, label: &str) -> Option<Arc<Mutex<Node>>> {
        for sub_node in &self.sub_nodes {
            if let Some(sub_node_label) = &sub_node.lock().unwrap().label {
                if sub_node_label == label {
                    return Some(sub_node.clone());
                }
            }
            let sub_node_with_label = sub_node.lock().unwrap().find_subnode_with_label(label);
            if sub_node_with_label.is_some() {
                return sub_node_with_label;
            }
        }
        None
    }

    pub fn find_subnode_with_path(&self, path: Vec<&str>) -> Option<Arc<Mutex<Node>>> {
        for sub_node in &self.sub_nodes {
            if sub_node.lock().unwrap().name == path[0] {
                if path.len() == 1 {
                    // Found the matching node
                    return Some(sub_node.clone());
                } else {
                    // There are more to match
                    let sub_node_with_path = sub_node
                        .lock()
                        .unwrap()
                        .find_subnode_with_path(path[1..].to_vec());
                    if sub_node_with_path.is_some() {
                        return sub_node_with_path;
                    }
                }
            }
        }
        None
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
    use crate::property::Property;

    #[test]
    fn test_node_empty() {
        let node = Node::new("node");
        assert_eq!(node.properties.len(), 0);
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
    fn test_node_properties() {
        let mut node = Node::new("node");
        node.add_property(Property::new_empty("prop0"));
        node.add_property(Property::new_u32("prop1", 42));
        assert_eq!(node.properties.len(), 2);
    }

    #[test]
    fn test_property_print() {
        let mut node = Node::new("node");
        node.add_property(Property::new_u32("prop", 42));
        let mut sub_node = Node::new("node");
        sub_node.add_property(Property::new_u32("prop", 12));
        node.add_sub_node(sub_node);

        let printing = format!("{}", node);
        assert_eq!(&printing, "node {\n\tprop = <0x0 0x0 0x0 0x2a>;\n\n\tnode {\n\t\tprop = <0x0 0x0 0x0 0xc>;\n\t};\n};\n");
    }
}
