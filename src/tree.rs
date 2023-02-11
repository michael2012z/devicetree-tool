// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dtb_generator::DtbGenerator;
use crate::dtb_parser::DtbParser;
use crate::dts_generator::DtsGenerator;
use crate::dts_parser::DtsParser;
use crate::node::Node;
use crate::reservation::Reservation;
use std::sync::{Arc, Mutex};

pub struct Tree {
    pub reservations: Vec<Arc<Mutex<Reservation>>>,
    pub root: Arc<Mutex<Node>>,
}

impl Tree {
    pub fn new(reservations: Vec<Reservation>, root: Node) -> Self {
        let mut reserv_refs = vec![];
        for r in reservations {
            reserv_refs.push(Arc::new(Mutex::new(r)));
        }
        Tree {
            reservations: reserv_refs,
            root: Arc::new(Mutex::new(root)),
        }
    }

    pub fn find_node_with_label(&self, label: &str) -> Option<Arc<Mutex<Node>>> {
        self.root.lock().unwrap().find_subnode_with_label(label)
    }

    pub fn find_node_with_path(&self, path: &str) -> Option<Arc<Mutex<Node>>> {
        let path: Vec<&str> = path.split("/").collect();
        if path.len() == 0 {
            Some(self.root.clone())
        } else {
            self.root
                .lock()
                .unwrap()
                .find_subnode_with_path(path[1..].to_vec())
        }
    }

    pub fn from_dts_bytes(dts: &[u8]) -> Self {
        DtsParser::from_bytes(&dts).parse()
    }

    pub fn generate_dts(&self) -> String {
        DtsGenerator::generate_tree(self)
    }
    pub fn from_dtb_bytes(dtb: &[u8]) -> Self {
        DtbParser::from_bytes(&dtb).parse()
    }

    pub fn generate_dtb(&self) -> Vec<u8> {
        let mut reservations = vec![];
        for reservation in &self.reservations {
            reservations.push(reservation.clone());
        }
        DtbGenerator::from_tree(&self.root.lock().unwrap(), reservations).generate()
    }
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = DtsGenerator::generate_tree(self);
        writeln!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::property::Property;

    #[test]
    fn test_tree_print() {
        let mut node = Node::new("/");
        node.add_property(Property::new_u32("prop", 42));
        node.add_sub_node(Node::new("sub_node"));
        let tree = Tree::new(vec![], node);

        let printing = format!("{}", tree);
        assert_eq!(
            &printing,
            "/dts-v1/;\n\n/ {\n\tprop = <0x0 0x0 0x0 0x2a>;\n\n\tsub_node {\n\t};\n};\n\n"
        );
    }
}
