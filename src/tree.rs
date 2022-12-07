// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::element::Element;
use crate::node::Node;

pub struct Tree {
    root: Node,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Tree { root }
    }
}

impl Element for Tree {
    fn to_dts(&self, _indent_level: u32) -> String {
        let mut dts = String::from("/dts-v1/;\n\n/ ");
        let root_dts = self.root.to_dts(0);
        dts.push_str(&root_dts);
        dts.push_str("\n");
        dts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;

    #[test]
    fn test_simple_tree() {
        let root = Node::new("root");
        let tree = Tree::new(root);
        println!("{}", tree.to_dts(0));
    }
}
