// Copyright (c) 2023, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dtb_generator::DtbGenerator;
use crate::dtb_parser::DtbParser;
use crate::dts_generator::DtsGenerator;
use crate::dts_parser::DtsParser;
use crate::node::Node;
use crate::reservation::Reservation;
use std::sync::{Arc, Mutex};

/// `Tree` contains everything of a device tree.
///
/// A `Tree` struct consists of:
///   - The root node of the device tree (mandatory)
///   - And the memory reservation blocks (optional)
pub struct DeviceTree {
    pub reservations: Vec<Arc<Mutex<Reservation>>>,
    pub root: Arc<Mutex<Node>>,
}

impl DeviceTree {
    /// Create a new device tree with a vector of reservation block and the root node.
    /// If there is not any reservation block, the vector should be empty.
    ///
    /// Example:
    ///
    /// ```
    /// use devicetree_tool::Reservation;
    /// use devicetree_tool::Node;
    /// use devicetree_tool::DeviceTree;
    ///
    /// let tree = DeviceTree::new(vec![], Node::new(""));
    ///
    /// assert_eq!(format!("{}", tree), "/dts-v1/;\n\n/ {\n};\n\n");
    /// ```
    pub fn new(reservations: Vec<Reservation>, root: Node) -> Self {
        let mut reserv_refs = vec![];
        for r in reservations {
            reserv_refs.push(Arc::new(Mutex::new(r)));
        }
        DeviceTree {
            reservations: reserv_refs,
            root: Arc::new(Mutex::new(root)),
        }
    }

    /// Find a 'Node' by label.
    ///
    /// Example:
    ///
    /// ```
    /// use devicetree_tool::Node;
    /// use devicetree_tool::DeviceTree;
    ///
    /// let mut root = Node::new("");
    ///
    /// // Add some nodes
    /// root.add_sub_node(Node::new_with_label("node1", "label1"));
    /// root.add_sub_node(Node::new_with_label("node2", "label2"));
    ///
    /// let tree = DeviceTree::new(vec![], root);
    ///
    /// // Find the nodes by their labels
    /// let node1 = tree.find_node_by_label("label1").unwrap();
    /// assert_eq!(node1.lock().unwrap().name, "node1");
    ///
    /// let node2 = tree.find_node_by_label("label2").unwrap();
    /// assert_eq!(node2.lock().unwrap().name, "node2");
    /// ```
    pub fn find_node_by_label(&self, label: &str) -> Option<Arc<Mutex<Node>>> {
        self.root.lock().unwrap().find_subnode_by_label(label)
    }

    /// Find a 'Node' by path.
    ///
    /// Example:
    ///
    /// ```
    /// use devicetree_tool::Node;
    /// use devicetree_tool::DeviceTree;
    ///
    /// let mut root = Node::new("");
    ///
    /// // Create a node with sub node
    /// let mut node_l1 = Node::new("node_l1");
    /// node_l1.add_sub_node(Node::new("node_l2"));
    ///
    /// root.add_sub_node(node_l1);
    ///
    /// let tree = DeviceTree::new(vec![], root);
    ///
    /// let node_l2 = tree.find_node_by_path("/node_l1/node_l2").unwrap();
    ///
    /// assert_eq!(node_l2.lock().unwrap().name, "node_l2");
    /// ```
    pub fn find_node_by_path(&self, path: &str) -> Option<Arc<Mutex<Node>>> {
        let path: Vec<&str> = path.split("/").collect();
        if path.len() == 0 {
            Some(self.root.clone())
        } else {
            self.root
                .lock()
                .unwrap()
                .find_subnode_by_path(path[1..].to_vec())
        }
    }

    /// Create a `Tree` from DTS text byte array.
    pub fn from_dts_bytes(dts: &[u8]) -> Self {
        DtsParser::from_bytes(&dts).parse()
    }

    /// Generate the DTS text of a `Tree`.
    pub fn generate_dts(&self) -> String {
        DtsGenerator::generate_tree(self)
    }

    /// Create a `Tree` from DTB binary byte array.
    pub fn from_dtb_bytes(dtb: &[u8]) -> Self {
        DtbParser::from_bytes(&dtb).parse()
    }

    /// Generate the DTB binary of a `Tree`.
    pub fn generate_dtb(&self) -> Vec<u8> {
        let mut reservations = vec![];
        for reservation in &self.reservations {
            reservations.push(reservation.clone());
        }
        DtbGenerator::from_tree(&self.root.lock().unwrap(), reservations).generate()
    }
}

impl std::fmt::Display for DeviceTree {
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
        let tree = DeviceTree::new(vec![], node);

        let printing = format!("{}", tree);
        assert_eq!(
            &printing,
            "/dts-v1/;\n\n/ {\n\tprop = <0x0 0x0 0x0 0x2a>;\n\n\tsub_node {\n\t};\n};\n\n"
        );
    }
}
