// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dtb_generator::DtbGenerator;
use crate::dtb_parser::DtbParser;
use crate::dts_generator::DtsGenerator;
use crate::dts_parser::DtsParser;
use crate::node::Node;
use crate::reservation::Reservation;

pub struct Tree {
    pub reservations: Vec<Reservation>,
    pub root: Node,
}

impl Tree {
    pub fn new(reservations: Vec<Reservation>, root: Node) -> Self {
        Tree {
            reservations,
            root: root,
        }
    }

    pub fn from_dts_bytes(dts: &[u8]) -> Self {
        DtsParser::parse(&dts)
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
            reservations.push(reservation.to_owned());
        }
        DtbGenerator::from_tree(&self.root, reservations).generate()
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
    use crate::attribute::Attribute;
    use crate::node::Node;

    #[test]
    fn test_tree_print() {
        let mut node = Node::new("/");
        node.add_attr(Attribute::new_u32("attr", 42));
        node.add_sub_node(Node::new("sub_node"));
        let tree = Tree::new(vec![], node);

        let printing = format!("{}", tree);
        assert_eq!(
            &printing,
            "/dts-v1/;\n\n/ {\n\tattr = <0x0 0x0 0x0 0x2a>;\n\n\tsub_node {\n\t};\n};\n\n"
        );
    }
}
