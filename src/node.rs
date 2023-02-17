// Copyright (c) 2023, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dts_generator::DtsGenerator;
use crate::property::Property;
use std::sync::{Arc, Mutex};

/// A node that is used to describe a device.
///
/// A node has a list of properties that are represented with a vector of `Property`.
///
/// `Node` can also contain other nodes.
pub struct Node {
    pub name: String,
    pub label: Option<String>,
    pub properties: Vec<Arc<Mutex<Property>>>,
    pub sub_nodes: Vec<Arc<Mutex<Node>>>,
}

impl Node {
    /// Create an empty `Node` with name.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::node::Node;
    ///
    /// let node = Node::new("node");
    ///
    /// assert_eq!(format!("{}", node), "node {\n};\n");
    /// ```
    pub fn new(name: &str) -> Self {
        Node {
            name: String::from(name),
            label: None,
            properties: Vec::new(),
            sub_nodes: Vec::new(),
        }
    }

    /// Create an empty `Node` with name and label.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::node::Node;
    ///
    /// let node = Node::new_with_label("node", "label");
    ///
    /// assert_eq!(format!("{}", node), "label: node {\n};\n");
    /// ```
    pub fn new_with_label(name: &str, label: &str) -> Self {
        Node {
            name: String::from(name),
            label: Some(String::from(label)),
            properties: Vec::new(),
            sub_nodes: Vec::new(),
        }
    }

    /// Add a `Property` to the `Node`.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::node::Node;
    /// use devicetree_tool::property::Property;
    ///
    /// let mut node = Node::new("node");
    ///
    /// node.add_property(Property::new_u32("prop1", 42));
    /// node.add_property(Property::new_str("prop2", "hello"));
    ///
    /// assert_eq!(node.properties.len(), 2);
    ///
    /// assert_eq!(format!("{}", node),
    ///            "node {\n\tprop1 = <0x0 0x0 0x0 0x2a>;\n\t\
    ///            prop2 = <0x68 0x65 0x6c 0x6c 0x6f 0x0>;\n};\n");
    /// ```
    pub fn add_property(&mut self, prop: Property) {
        self.properties.push(Arc::new(Mutex::new(prop)));
    }

    /// Add a sub node to the `Node`.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::node::Node;
    /// use devicetree_tool::property::Property;
    ///
    /// let mut node = Node::new("node");
    ///
    /// // Create a sub node
    /// let mut sub_node = Node::new("sub_node");
    /// sub_node.add_property(Property::new_u32("prop", 42));
    ///
    /// // Add the sub node
    /// node.add_sub_node(sub_node);
    ///
    /// assert_eq!(node.sub_nodes.len(), 1);
    /// assert_eq!(format!("{}", node),
    ///            "node {\n\n\tsub_node {\n\t\tprop = <0x0 0x0 0x0 0x2a>;\n\t};\n};\n");
    /// ```
    pub fn add_sub_node(&mut self, sub_node: Node) {
        self.sub_nodes.push(Arc::new(Mutex::new(sub_node)));
    }

    /// Find `Property` from a `Node` by name.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::node::Node;
    /// use devicetree_tool::property::Property;
    ///
    /// let mut node = Node::new("node");
    ///
    /// // Now the node hasn't any property
    /// assert_eq!(node.find_property("prop").is_none(), true);
    ///
    /// // Add a property
    /// node.add_property(Property::new_u32("prop", 42));
    ///
    /// // Find the property from the node
    /// let prop = node.find_property("prop").unwrap();
    ///
    /// assert_eq!(prop.lock().unwrap().value, vec![0u8, 0u8, 0u8, 42u8]);
    /// ```
    pub fn find_property(&self, name: &str) -> Option<Arc<Mutex<Property>>> {
        for prop in &self.properties {
            if name == prop.lock().unwrap().name {
                return Some(prop.clone());
            }
        }
        None
    }

    /// Find sub node from a `Node` by name.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::node::Node;
    /// use devicetree_tool::property::Property;
    ///
    /// let mut node = Node::new("node");
    ///
    /// // Now the node hasn't any sub node
    /// assert_eq!(node.find_subnode_by_name("subnode").is_none(), true);
    ///
    /// // Add a sub node
    /// node.add_sub_node(Node::new("subnode"));
    ///
    /// // Find the sub node from the node
    /// let sub_node = node.find_subnode_by_name("subnode").unwrap();
    ///
    /// assert_eq!(sub_node.lock().unwrap().name, "subnode");
    /// ```
    pub fn find_subnode_by_name(&self, name: &str) -> Option<Arc<Mutex<Node>>> {
        for sub_node in &self.sub_nodes {
            if sub_node.lock().unwrap().name == name {
                return Some(sub_node.clone());
            }
        }
        None
    }

    /// Find sub node from a `Node` by label.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::node::Node;
    /// use devicetree_tool::property::Property;
    ///
    /// let mut node = Node::new("node");
    ///
    /// // Now the node hasn't any sub node
    /// assert_eq!(node.find_subnode_by_label("label").is_none(), true);
    ///
    /// // Add a sub node
    /// node.add_sub_node(Node::new_with_label("subnode", "label"));
    ///
    /// // Find the sub node from the node
    /// let sub_node = node.find_subnode_by_label("label").unwrap();
    ///
    /// assert_eq!(sub_node.lock().unwrap().name, "subnode");
    /// ```
    pub fn find_subnode_by_label(&self, label: &str) -> Option<Arc<Mutex<Node>>> {
        for sub_node in &self.sub_nodes {
            if let Some(sub_node_label) = &sub_node.lock().unwrap().label {
                if sub_node_label == label {
                    return Some(sub_node.clone());
                }
            }
            let sub_node_with_label = sub_node.lock().unwrap().find_subnode_by_label(label);
            if sub_node_with_label.is_some() {
                return sub_node_with_label;
            }
        }
        None
    }

    /// Find sub node from a `Node` by path.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::node::Node;
    /// use devicetree_tool::property::Property;
    ///
    /// let mut node_layer_1 = Node::new("node_layer_1");
    ///
    /// // Now the node hasn't any sub node
    /// assert_eq!(node_layer_1.find_subnode_by_path(vec![ "node_layer_2", "node_layer_3"]).is_none(), true);
    ///
    /// // Create a layer-2 sub node
    /// let mut node_layer_2 = Node::new("node_layer_2");
    ///
    /// // Add a layer-3 sub node
    /// node_layer_2.add_sub_node(Node::new("node_layer_3"));
    ///
    /// node_layer_1.add_sub_node(node_layer_2);
    ///
    /// // Find the layer-3 sub node
    /// let sub_node = node_layer_1.find_subnode_by_path(vec!["node_layer_2", "node_layer_3"]).unwrap();
    ///
    /// assert_eq!(sub_node.lock().unwrap().name, "node_layer_3");
    /// ```
    pub fn find_subnode_by_path(&self, path: Vec<&str>) -> Option<Arc<Mutex<Node>>> {
        for sub_node in &self.sub_nodes {
            if sub_node.lock().unwrap().name == path[0] {
                if path.len() == 1 {
                    // Found the matching node
                    return Some(sub_node.clone());
                } else {
                    // There are more to match
                    let sub_node_with_path = sub_node
                        .lock()
                        .unwrap()
                        .find_subnode_by_path(path[1..].to_vec());
                    if sub_node_with_path.is_some() {
                        return sub_node_with_path;
                    }
                }
            }
        }
        None
    }
}

impl std::fmt::Display for Node {
    /// Print a `Property` in the format of DTS
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = DtsGenerator::generate_node(self, 0);
        writeln!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::Property;

    #[test]
    fn test_node_empty() {
        let node = Node::new("node");
        assert_eq!(node.properties.len(), 0);
        assert_eq!(node.sub_nodes.len(), 0);
    }

    #[test]
    fn test_node_sub_nodes() {
        let mut node = Node::new("node");
        node.add_sub_node(Node::new("sub_node_0"));
        node.add_sub_node(Node::new("sub_node_1"));
        node.add_sub_node(Node::new("sub_node_2"));

        let mut sub_node_3 = Node::new("sub_node_3");
        sub_node_3.add_sub_node(Node::new("sub_node_30"));
        sub_node_3.add_sub_node(Node::new("sub_node_31"));

        assert_eq!(sub_node_3.sub_nodes.len(), 2);

        node.add_sub_node(sub_node_3);
        assert_eq!(node.sub_nodes.len(), 4);
    }

    #[test]
    fn test_node_properties() {
        let mut node = Node::new("node");
        node.add_property(Property::new_empty("prop0"));
        node.add_property(Property::new_u32("prop1", 42));
        assert_eq!(node.properties.len(), 2);
    }

    #[test]
    fn test_property_print() {
        let mut node = Node::new("node");
        node.add_property(Property::new_u32("prop", 42));
        let mut sub_node = Node::new("node");
        sub_node.add_property(Property::new_u32("prop", 12));
        node.add_sub_node(sub_node);

        let printing = format!("{}", node);
        assert_eq!(&printing, "node {\n\tprop = <0x0 0x0 0x0 0x2a>;\n\n\tnode {\n\t\tprop = <0x0 0x0 0x0 0xc>;\n\t};\n};\n");
    }

    #[test]
    fn test_find_subnode_by_path() {
        let mut node_layer_1 = Node::new("node_layer_1");

        assert_eq!(
            node_layer_1
                .find_subnode_by_path(vec!["node_layer_2", "node_layer_3"])
                .is_none(),
            true
        );

        let mut node_layer_2 = Node::new("node_layer_2");
        node_layer_2.add_sub_node(Node::new("node_layer_3"));
        node_layer_1.add_sub_node(node_layer_2);

        let sub_node = node_layer_1
            .find_subnode_by_path(vec!["node_layer_2", "node_layer_3"])
            .unwrap();
        assert_eq!(sub_node.lock().unwrap().name, "node_layer_3");
    }
}
