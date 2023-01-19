// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::{attribute::Attribute, node::Node, reservation::Reservation, tree::Tree, utils::Utils};

pub struct DtsGenerator {}

impl DtsGenerator {
    pub fn generate_attribute(attribute: &Attribute, indent_level: u32) -> String {
        let mut s = String::from(format!("{}{}", Utils::indent(indent_level), attribute.name));
        if attribute.value.len() > 0 {
            s.push_str(" = <");
            for i in 0..attribute.value.len() {
                let d = attribute.value[i];
                if i > 0 {
                    s.push(' ')
                }
                s.push_str(&format!("{:#x}", d));
            }
            s.push_str(">;");
        } else {
            s.push_str(";");
        }
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
        for attr in &node.attributes {
            s.push_str(&DtsGenerator::generate_attribute(
                &attr.clone().lock().unwrap(),
                indent_level + 1,
            ));
            s.push_str("\n");
        }

        for sub_node in node.sub_nodes.iter() {
            s.push_str("\n");
            s.push_str(&DtsGenerator::generate_node(
                &sub_node.clone().lock().unwrap(),
                indent_level + 1,
            ));
            s.push_str("\n");
        }
        s.push_str(&format!("{indents}}};"));
        s
    }

    pub fn generate_reservation(reservation: &Reservation, _indent_level: u32) -> String {
        String::from(format!(
            "/memreserve/ {:#018x} {:#018x};",
            reservation.address, reservation.length
        ))
    }

    pub fn generate_tree(tree: &Tree) -> String {
        let mut dts = String::from("/dts-v1/;\n\n");
        if tree.reservations.len() > 0 {
            for reservation in &tree.reservations {
                let reserv = reservation.lock().unwrap();
                let reservation_dts = DtsGenerator::generate_reservation(&reserv, 0);
                dts.push_str(&reservation_dts);
                dts.push_str("\n");
            }
            dts.push_str("\n");
        }
        let root_dts = DtsGenerator::generate_node(&tree.root.clone().lock().unwrap(), 0);
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
        let attr = Attribute::new_empty("attr");
        assert_eq!(DtsGenerator::generate_attribute(&attr, 0), "attr;");
    }

    #[test]
    fn test_dts_generate_attribute_u32() {
        let attr = Attribute::new_u32("attr", 42);
        assert_eq!(
            DtsGenerator::generate_attribute(&attr, 0),
            "attr = <0x0 0x0 0x0 0x2a>;"
        );
    }

    #[test]
    fn test_dts_generate_attribute_strs() {
        let string1 = String::from("hello");
        let string2 = String::from("abc");
        let strs = vec![string1, string2];
        let attr = Attribute::new_strings("attr", strs);
        assert_eq!(
            DtsGenerator::generate_attribute(&attr, 0),
            "attr = <0x68 0x65 0x6c 0x6c 0x6f 0x0 0x61 0x62 0x63 0x0>;"
        );
    }

    #[test]
    fn test_dts_generate_attribute_str() {
        let s = String::from("hello abc");
        let attr = Attribute::new_string("attr", s);
        assert_eq!(
            DtsGenerator::generate_attribute(&attr, 0),
            "attr = <0x68 0x65 0x6c 0x6c 0x6f 0x20 0x61 0x62 0x63 0x0>;"
        );
    }

    #[test]
    fn test_dts_generate_attribute_bytes() {
        let bytes = vec![0u8, 1u8, 2u8, 3u8];
        let attr = Attribute::new_u8s("attr", bytes);
        assert_eq!(
            DtsGenerator::generate_attribute(&attr, 0),
            "attr = <0x0 0x1 0x2 0x3>;"
        );
    }

    #[test]
    fn test_dts_generate_node() {
        let attr = Attribute::new_u32("attr", 42);
        let mut node = Node::new("node");
        node.add_attr(attr);
        assert_eq!(
            DtsGenerator::generate_node(&node, 0),
            "node {\n\tattr = <0x0 0x0 0x0 0x2a>;\n};"
        );
    }

    #[test]
    fn test_dts_generate_sub_node() {
        let attr = Attribute::new_u32("attr1", 42);
        let mut node = Node::new("node");
        node.add_attr(attr);

        let mut sub_node = Node::new("sub_node");
        sub_node.add_attr(Attribute::new_u32("attr2", 99));

        node.add_sub_node(sub_node);
        assert_eq!(
            DtsGenerator::generate_node(&node, 0),
            "node {\n\tattr1 = <0x0 0x0 0x0 0x2a>;\n\n\tsub_node {\n\t\tattr2 = <0x0 0x0 0x0 0x63>;\n\t};\n};"
        );
    }

    #[test]
    fn test_dts_generate_reservation() {
        let reservation = Reservation::new(0x100000, 0x200000);
        assert_eq!(
            DtsGenerator::generate_reservation(&reservation, 0),
            "/memreserve/ 0x0000000000100000 0x0000000000200000;"
        );
    }

    #[test]
    fn test_dts_generate_tree_simple() {
        let root = Node::new("root");
        let tree = Tree::new(vec![], root);
        assert_eq!(
            DtsGenerator::generate_tree(&tree),
            "/dts-v1/;\n\nroot {\n};\n"
        );
    }

    #[test]
    fn test_dts_generate_tree_reservation() {
        let root = Node::new("root");
        let reservation = Reservation::new(0x0, 0x100000);
        let tree = Tree::new(vec![reservation], root);
        assert_eq!(
            DtsGenerator::generate_tree(&tree),
            "/dts-v1/;\n\n/memreserve/ 0x0000000000000000 0x0000000000100000;\n\nroot {\n};\n"
        );
    }
}
