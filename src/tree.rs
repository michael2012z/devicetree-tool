// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

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
}
