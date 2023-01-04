// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dtb_generator::DtbGenerator;
use crate::dtb_parser::DtbParser;
use crate::dts_generator::DtsGenerator;
use crate::dts_parser::DtsParser;
use crate::node::Node;
use std::rc::Rc;

pub struct Tree {
    pub root: Rc<Node>,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Tree {
            root: Rc::new(root),
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
        DtbGenerator::from_tree(self.root.clone()).generate()
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
        let tree = Tree::new(node);

        let printing = format!("{}", tree);
        assert_eq!(
            &printing,
            "/dts-v1/;\n\n/ {\n\tattr = <0x0 0x0 0x0 0x2a>;\n\n\tsub_node {\n\t};\n};\n\n"
        );
    }
}
