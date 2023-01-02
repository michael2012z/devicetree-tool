// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

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

    pub fn from_dtb_bytes(dtb: &[u8]) -> Self {
        DtbParser::from_bytes(&dtb).parse()
    }
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = DtsGenerator::generate_tree(self);
        writeln!(f, "{s}")
    }
}
