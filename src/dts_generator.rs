// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::{attribute::Attribute, node::Node, tree::Tree, utils::Utils};

pub struct DtsGenerator {}

impl DtsGenerator {
    pub fn generate_attribute(attribute: &Attribute, indent_level: u32) -> String {
        let mut s = String::from(format!(
            "{}{} = <",
            Utils::indent(indent_level),
            attribute.name
        ));
        for i in 0..attribute.value.len() {
            let d = attribute.value[i];
            if i > 0 {
                s.push(' ')
            }
            s.push_str(&format!("{:#x}", d));
        }
        s.push_str(">;");
        s
    }

    pub fn generate_node(node: &Node, indent_level: u32) -> String {
        let mut s = String::new();
        let indents = Utils::indent(indent_level);
        s.push_str(&format!("{indents}"));
        if node.name.len() > 0 {
            s.push_str(&format!("{} ", node.name));
        }
        s.push_str("{\n");
        for attr in node.attributes.iter() {
            s.push_str(&DtsGenerator::generate_attribute(&attr, indent_level + 1));
            s.push_str("\n");
        }

        for sub_node in node.sub_nodes.iter() {
            s.push_str("\n");
            s.push_str(&DtsGenerator::generate_node(&sub_node, indent_level + 1));
            s.push_str("\n");
        }
        s.push_str(&format!("{indents}}};"));
        s
    }

    pub fn generate_tree(tree: &Tree) -> String {
        let mut dts = String::from("/dts-v1/;\n\n/ ");
        let root_dts = DtsGenerator::generate_node(&tree.root, 0);
        dts.push_str(&root_dts);
        dts.push_str("\n");
        dts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dts_generate_attribute_none() {
        let attr = Attribute::new_empty("name");
        println!("{}", DtsGenerator::generate_attribute(&attr, 0));
    }

    #[test]
    fn test_dts_generate_attribute_u32() {
        let attr = Attribute::new_u32("name", 42);
        println!("{}", DtsGenerator::generate_attribute(&attr, 0));
    }

    #[test]
    fn test_dts_generate_attribute_strs() {
        let string1 = String::from("&str abc");
        let string2 = String::from("def");
        let strs = vec![string1, string2];
        let attr = Attribute::new_strings("name", strs);
        println!("{}", DtsGenerator::generate_attribute(&attr, 0));
    }

    #[test]
    fn test_dts_generate_attribute_str() {
        let s = String::from("&str abc");
        let attr = Attribute::new_string("name", s);
        println!("{}", DtsGenerator::generate_attribute(&attr, 0));
    }

    #[test]
    fn test_dts_generate_attribute_bytes() {
        let bytes = vec![0u8, 1u8, 2u8, 3u8];
        let attr = Attribute::new_u8s("name", bytes);
        println!("{}", DtsGenerator::generate_attribute(&attr, 0));
    }

    #[test]
    fn test_simple_node() {
        let attr = Attribute::new_u32("attr1", 42);
        let mut node = Node::new("node");
        node.add_attr(attr);
        println!("Node: {}", DtsGenerator::generate_node(&node, 0));
    }

    #[test]
    fn test_sub_node() {
        let attr = Attribute::new_u32("attr1", 42);
        let mut node = Node::new("node1");
        node.add_attr(attr);

        let mut sub_node = Node::new("node2");
        sub_node.add_attr(Attribute::new_u32("attr4", 99));

        node.add_sub_node(sub_node);
        println!("Node: {}", DtsGenerator::generate_node(&node, 0));
    }

    #[test]
    fn test_simple_tree() {
        let root = Node::new("root");
        let tree = Tree::new(root);
        println!("{}", DtsGenerator::generate_tree(&tree));
    }
}
