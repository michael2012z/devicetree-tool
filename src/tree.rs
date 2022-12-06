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
    fn to_dts(&self) -> String {
        self.root.to_dts()
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
        println!("{}", tree.to_dts());
    }
}
