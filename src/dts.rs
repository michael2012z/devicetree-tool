// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

pub struct Dts {}

#[cfg(test)]
mod tests {
    use crate::attribute::Attribute;
    use crate::element::Element;
    use crate::node::Node;
    use crate::tree::Tree;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_dts_0() {
        // Read the DTS text from test data folder
        let dts_0_text = std::fs::read_to_string("test/dts_0.dts").unwrap();
        println!("{dts_0_text}");
        // Build the same device tree with API and compare
        let mut root = Node::new("");
        root.add_attr(Arc::new(Mutex::new(Attribute::new(
            "compatible",
            vec![String::from("linux,dummy-virt")],
        ))));
        root.add_attr(Arc::new(Mutex::new(Attribute::new("#address-cells", 2u32))));
        root.add_attr(Arc::new(Mutex::new(Attribute::new("#size-cells", 2u32))));
        root.add_attr(Arc::new(Mutex::new(Attribute::new(
            "interrupt-parent",
            1u32,
        ))));
        let dt = Tree::new(root);
        let dts = dt.to_dts(0);
        assert_eq!(dts_0_text, dts);
    }

    #[test]
    fn test_dts_1() {
        // Read the DTS text from test data folder
        let dts_1_text = std::fs::read_to_string("test/dts_1.dts").unwrap();
        println!("{dts_1_text}");
        // Build the same device tree with API and compare
        let mut root = Node::new("");
        root.add_attr(Arc::new(Mutex::new(Attribute::new("#address-cells", 2u32))));
        root.add_attr(Arc::new(Mutex::new(Attribute::new("#size-cells", 2u32))));
        let mut memory = Node::new("memory");
        memory.add_attr(Arc::new(Mutex::new(Attribute::new(
            "device_type",
            String::from("memory"),
        ))));
        let reg = vec![0u32, 0x40000000u32, 1u32, 0u32];
        memory.add_attr(Arc::new(Mutex::new(Attribute::new("reg", reg))));
        root.add_sub_node(memory);
        let dt = Tree::new(root);
        let dts = dt.to_dts(0);
        assert_eq!(dts_1_text, dts);
    }
}
