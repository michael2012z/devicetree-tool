// Copyright (c) 2023, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::{devicetree::DeviceTree, node::Node, property::Property, reservation::Reservation};
use std::sync::{Arc, Mutex};

pub struct DtsParser {
    dts: Vec<u8>,
    next_phandle: u32,
    tree: DeviceTree,
}

impl DtsParser {
    pub fn from_bytes(dts: &[u8]) -> Self {
        DtsParser {
            dts: dts.to_owned(),
            next_phandle: 0,
            tree: DeviceTree::new(vec![], Node::new("/")),
        }
    }

    pub fn parse(&mut self) -> DeviceTree {
        // Pre-process to remove comments and handle inclusion
        let dts_string = String::from_utf8_lossy(&self.dts);
        let dts_string = DtsParser::pre_process(&dts_string, 8).unwrap_or(dts_string.into());
        let dts = dts_string.as_bytes();

        self.parse_tree(dts, true);
        self.parse_tree(dts, false);

        let mut reservations_clone = vec![];
        for reservation in &self.tree.reservations {
            reservations_clone.push(reservation.clone());
        }
        DeviceTree {
            reservations: reservations_clone,
            root: self.tree.root.clone(),
        }
    }

    // Parse the DTS text that has been pre-processed and update the tree struct.
    // If `node_only` is true, only parse the node structure, and create nodes and subnodes
    // in the tree with names, all properties and indirectives will be ignored.
    fn parse_tree(&mut self, dts: &[u8], node_only: bool) {
        let root_node = self.tree.root.clone();
        let mut i: usize = 0;
        let mut text: Vec<u8> = vec![];
        while i < dts.len() {
            match dts[i] as char {
                ';' => {
                    // On the top level of a DTS, the semicolon may conclude one of: "/dts-v1/" or "/memreserve/"
                    let statement =
                        &String::from(String::from_utf8_lossy(&text).to_string().trim());
                    i = i + 1;
                    text.clear();

                    if node_only {
                        continue;
                    }

                    if statement == "/dts-v1/" {
                        println!("detected /dts-v1/;");
                    } else if statement.starts_with("/memreserve/") {
                        let mut reservation = statement.split_ascii_whitespace();
                        let _ = reservation.next().unwrap();
                        let address = reservation.next().unwrap();
                        let address = if address.starts_with("0x") {
                            u64::from_str_radix(&address[2..], 16).unwrap()
                        } else {
                            u64::from_str_radix(address, 10).unwrap()
                        };
                        let length = reservation.next().unwrap();
                        let length = if length.starts_with("0x") {
                            u64::from_str_radix(&length[2..], 16).unwrap()
                        } else {
                            u64::from_str_radix(length, 10).unwrap()
                        };
                        println!(
                            "detected /memreserve/: address = {:#018x}, length = {:#018x}",
                            address, length
                        );
                        self.tree
                            .reservations
                            .push(Arc::new(Mutex::new(Reservation::new(address, length))));
                    } else {
                        panic!("unknown top-level statement: {statement}");
                    }
                }
                '{' => {
                    // Found node
                    let node_name =
                        &String::from(String::from_utf8_lossy(&text).to_string().trim());

                    // The node name must be "/", fail otherwise
                    if node_name != "/" {
                        panic!("node {node_name} is not expected")
                    }

                    i = i + 1;
                    // Update the root node content
                    let node_size = self.parse_node(&dts[i..], root_node.clone(), node_only).unwrap_or(0);
                    i = i + node_size;
                    text.clear();
                }
                _ => {
                    text.push(dts[i]);
                    i = i + 1;
                }
            }
        }
    }

    fn parse_node(
        &mut self,
        dts: &[u8],
        node: Arc<Mutex<Node>>,
        node_only: bool,
    ) -> Result<usize, String> {
        let mut i: usize = 0;
        let mut text: Vec<u8> = vec![];
        let mut at_end = false;
        while i < dts.len() {
            match dts[i] as char {
                '{' => {
                    // Found node
                    let sub_node_name =
                        &String::from(String::from_utf8_lossy(&text).to_string().trim());
                    println!("found node {sub_node_name}");

                    let (label, sub_node_name) = if sub_node_name.contains(":") {
                        let parts: Vec<&str> = sub_node_name.split(":").collect();
                        (Some(String::from(parts[0])), String::from(parts[1]))
                    } else {
                        (None, String::from(sub_node_name.trim()))
                    };

                    // If a sub_node with the name doesn't exist, create one
                    if !node
                        .lock()
                        .map_err(|err| format!("node lock: {err}"))?
                        .sub_nodes
                        .iter()
                        .any(|x| match x.lock() {
                            Ok(n) => n.name == sub_node_name,
                            Err(_) => false,
                        })
                    {
                        let new_sub_node = if let Some(label) = label {
                            Node::new_with_label(&sub_node_name, &label)
                        } else {
                            Node::new(&sub_node_name)
                        };

                        if let Ok(mut n) = node.lock() {
                            n.add_sub_node(new_sub_node);
                        }
                    }

                    // Get the sub_node out and update
                    let sub_node = node
                        .lock()
                        .map_err(|err| format!("node lock: {err}"))?
                        .find_subnode_by_name(&sub_node_name)
                        .ok_or(format!("no sub node: {sub_node_name}"))?;

                    i = i + 1;
                    let node_size = self.parse_node(&dts[i..], sub_node, node_only).unwrap_or(0);
                    i = i + node_size;
                    text.clear();
                }
                '}' => {
                    // Come to the end of current node, expecting a ';' to finish
                    at_end = true;
                    i = i + 1;
                }
                '=' => {
                    // Found a property with value
                    let prop_name =
                        &String::from(String::from_utf8_lossy(&text).to_string().trim());
                    println!("found property {prop_name} with value:");
                    i = i + 1;
                    let (property_value_size, property_value) =
                        self.parse_property_value(&dts[i..], node_only)?;
                    i = i + property_value_size;
                    text.clear();
                    if !node_only {
                        let prop = Property::new_u8s(prop_name, property_value);
                        node.lock().map_err(|err| format!("node lock: {err}"))?.add_property(prop);
                    }
                }
                ';' => {
                    // We found either:
                    //  - A property without value or a compiler directive
                    //  - A directive like `/delete-node/` or `/delete-property/`
                    //  - Or the end of the node
                    i = i + 1;
                    if at_end {
                        return Ok(i);
                    } else {
                        // A property without value or a comipler directive
                        let prop_name =
                            String::from(String::from_utf8_lossy(&text).to_string().trim());
                        text.clear();

                        if node_only {
                            continue;
                        }

                        if prop_name.starts_with("/") {
                            // A compiler directive
                            let directive = prop_name;
                            println!("found directive: {directive}");
                            let mut slices = directive.split_ascii_whitespace();
                            let instruction = slices.next().unwrap();
                            if instruction == "/delete-node/" {
                                let sub_node_name = slices.next().unwrap();
                                println!("delete node: {sub_node_name}");
                                let sub_node_index = node
                                    .lock()
                                    .map_err(|err| format!("node lock: {err}"))?
                                    .sub_nodes
                                    .iter()
                                    .position(|x| x.lock().unwrap().name == sub_node_name)
                                    .ok_or(format!("missing sub node: {sub_node_name}"))?;
                                node
                                    .lock()
                                    .map_err(|err| format!("node lock: {err}"))?
                                    .sub_nodes
                                    .remove(sub_node_index);
                            } else if instruction == "/delete-property/" {
                                let property_name = slices.next().unwrap();
                                println!("delete property: {property_name}");
                                let property_index = node
                                    .lock()
                                    .map_err(|err| format!("node lock: {err}"))?
                                    .properties
                                    .iter()
                                    .position(|x| x.lock().unwrap().name == property_name)
                                    .ok_or(format!("missing property: {property_name}"))?;
                                node
                                    .lock()
                                    .map_err(|err| format!("node lock: {err}"))?
                                    .properties
                                    .remove(property_index);
                            } else {
                                return Err(format!("unknown comipler directive {directive}"));
                            }
                        } else {
                            println!("found property {} without value", prop_name);
                            let prop = Property::new_empty(&prop_name);
                            node.lock().map_err(|err| format!("node lock: {err}"))?.add_property(prop);
                        }
                    }
                }
                _ => {
                    text.push(dts[i]);
                    i = i + 1;
                }
            }
        }
        Err("property not ended".into())
    }

    fn parse_property_value(
        &mut self,
        dts: &[u8],
        ignore_content: bool,
    ) -> Result<(usize, Vec<u8>), String> {
        let mut value: Vec<u8> = vec![];
        let mut i: usize = 0;
        let mut text: Vec<u8> = vec![];

        // 3 types of value (piece) are possible:
        //  - 1. Cell (<...>)
        //  - 2. Byte sequence ([...])
        //  - 3. String ("...")
        let mut value_type = 0; // 0 for undetermined

        while i < dts.len() {
            match dts[i] as char {
                '<' => {
                    // Cell type
                    if value_type != 0 {
                        return Err(format!("found cell-start while parsing another property type {value_type}"));
                    }
                    value_type = 1;
                    text.clear();
                }
                '>' => {
                    if value_type != 1 {
                        return Err(format!("found cell-end while parsing another property type {value_type}"));
                    }
                    value_type = 0;

                    if ignore_content == false {
                        let cells_value = self.parse_property_value_cells(&text);
                        for d in cells_value {
                            value.push(d)
                        }
                    }
                    text.clear();
                }
                '[' => {
                    // Bytes type
                    if value_type != 0 {
                        return Err(format!("found bytes-start while parsing another property type {value_type}"));
                    }

                    value_type = 2;
                    text.clear();
                }
                ']' => {
                    if value_type != 2 {
                        return Err(format!("found bytes-end while parsing another property type {value_type}"));
                    }
                    value_type = 0;

                    if ignore_content == false {
                        let bytes_value = DtsParser::parse_property_value_bytes(&text);
                        for d in bytes_value {
                            value.push(d)
                        }
                    }
                    text.clear();
                }
                '"' => {
                    // Bytes type
                    if value_type == 0 {
                        // At the start of a string
                        value_type = 3;
                        text.clear();
                    } else if value_type == 3 {
                        // At the end of a string
                        value_type = 0;
                        if ignore_content == false {
                            let string_value = DtsParser::parse_property_value_string(&text);
                            for d in string_value {
                                value.push(d)
                            }
                        }
                        text.clear();
                    } else {
                        return Err(format!("found string while parsing another property type {value_type}"));
                    }
                }
                '\\' => {
                    // Met an escape char, push the esc char and the next char to buffer
                    text.push(dts[i]);
                    i = i + 1;
                    text.push(dts[i]);
                }
                ';' => {
                    // Conclude the property
                    // This is the only exit of the function
                    return Ok((i + 1, value));
                }
                _ => {
                    text.push(dts[i]);
                }
            }
            i = i + 1;
        }
        Err("property not ended".into())
    }

    fn parse_property_value_cells(&mut self, text: &[u8]) -> Vec<u8> {
        let mut value: Vec<u8> = vec![];
        println!("cells: {}", String::from_utf8_lossy(text));

        for num in String::from_utf8_lossy(text).split_whitespace() {
            // A value could be in format:
            //   * &LABEL
            //   * &{/FULL/PATH}
            //   * 0x12
            //   * 42
            let n = if num.starts_with("&") {
                // This is a reference to another node
                if num[1..].starts_with("{") && num[1..].ends_with("}") {
                    // Get the full path
                    let ref_node_path = &num[2..(num.len() - 1)];
                    let node_to_ref = match self.tree.find_node_by_path(ref_node_path) {
                        Some(n) => n,
                        None => continue,
                    };
                    let phandle_prop = match node_to_ref.lock() {
                        Ok(n) => n.find_property("phandle"),
                        Err(_) => continue,
                    };
                    if let Some(phandle_prop) = phandle_prop {
                        let value = match phandle_prop.lock() {
                            Ok(p) => [p.value[0], p.value[1], p.value[2], p.value[3]],
                            Err(_) => continue,
                        };
                        u32::from_be_bytes(value)
                    } else {
                        let phandle = self.next_phandle;
                        self.next_phandle = self.next_phandle + 1;
                        if let Ok(mut n) = node_to_ref.lock() {
                            n.add_property(Property::new_u32("phandle", phandle));
                        };
                        phandle
                    }
                } else {
                    // It should be a label
                    let label = &num[1..];
                    let node_to_ref = match self.tree.find_node_by_label(label) {
                        Some(n) => n,
                        None => continue,
                    };

                    let phandle_prop = match node_to_ref.lock() {
                        Ok(n) => n.find_property("phandle"),
                        Err(_) => continue,
                    };

                    if let Some(phandle_prop) = phandle_prop {
                        let value = match phandle_prop.lock() {
                            Ok(p) => [p.value[0], p.value[1], p.value[2], p.value[3]],
                            Err(_) => continue,
                        };
                        u32::from_be_bytes(value)
                    } else {
                        let phandle = self.next_phandle;
                        self.next_phandle = self.next_phandle + 1;
                        if let Ok(mut n) = node_to_ref.lock() {
                            n.add_property(Property::new_u32("phandle", phandle));
                        }
                        phandle
                    }
                }
            } else if num.starts_with("0x") {
                u32::from_str_radix(&num[2..], 16).unwrap_or(0)
            } else {
                u32::from_str_radix(num, 10).unwrap_or(0)
            };
            let n_u8_vec = n.to_be_bytes().to_vec();
            for n in n_u8_vec {
                value.push(n);
            }
            println!("{n:x}");
        }
        value
    }

    fn parse_property_value_bytes(text: &[u8]) -> Vec<u8> {
        let mut value: Vec<u8> = vec![];
        println!("bytes: {}", String::from_utf8_lossy(text));
        println!("bytes:");
        for num in String::from_utf8_lossy(text).split_whitespace() {
            let n = if num.starts_with("0x") {
                match u8::from_str_radix(&num[2..], 16) {
                    Ok(n) => n,
                    Err(_) => continue,
                }
            } else {
                match u8::from_str_radix(num, 10) {
                    Ok(n) => n,
                    Err(_) => continue,
                }
            };
            value.push(n);
            println!("{n:x}");
        }
        value
    }

    fn parse_property_value_string(text: &[u8]) -> Vec<u8> {
        println!("string: {}", String::from_utf8_lossy(text));
        let mut bytes = text.to_vec();
        // Append the terminator
        bytes.push(0);
        bytes
    }

    fn pre_process(dts: &str, inclusion_depth: usize) -> Result<String, String> {
        if inclusion_depth == 0 {
            Err(format!("maximum inclusion depth reached"))
        } else {
            let dts_bytes = dts.as_bytes();
            let dts_bytes = &DtsParser::remove_c_style_comments(dts_bytes);
            let dts_bytes = &DtsParser::remove_cpp_style_comments(dts_bytes);

            let dts = String::from_utf8_lossy(dts_bytes);

            let mut processed_dts = String::new();
            let lines: Vec<&str> = dts.split("\n").collect();
            for line in lines {
                if let Some(index) = line.find("/include/") {
                    if index > 0 {
                        // something is before the `/include/`
                        processed_dts.push_str(&line[0..index]);
                    }

                    let path = line[(index + 9)..].trim();
                    if !path.starts_with('"') || !path.ends_with('"') {
                        return Err(format!("included file path error: {path}"))
                    }
                    let path = &path[1..(path.len() - 1)];
                    println!("path: {path}");
                    let included_dts = std::fs::read_to_string(path).unwrap();
                    let included_dts = DtsParser::pre_process(&included_dts, inclusion_depth - 1)
                        .unwrap_or(included_dts);
                    processed_dts.push_str(&included_dts);
                    processed_dts.push('\n');
                } else {
                    processed_dts.push_str(line);
                    processed_dts.push('\n');
                }
            }
            Ok(processed_dts)
        }
    }

    // Return the space of a C-style comment: (start location, size)
    fn find_c_comment(text: &[u8]) -> Option<(usize, usize)> {
        if let Some(comment_start) = text
            .windows(2)
            .position(|window| window == &['/' as u8, '*' as u8])
        {
            if let Some(comment_end) = text[comment_start..]
                .windows(2)
                .position(|window| window == &['*' as u8, '/' as u8])
            {
                Some((comment_start, comment_end + 2))
            } else {
                panic!("Format error: C-style comments not enclosed")
            }
        } else {
            None
        }
    }

    // Return the space of a C-style comment: (start location, size)
    fn find_cpp_comment(text: &[u8]) -> Option<(usize, usize)> {
        if let Some(comment_start) = text
            .windows(2)
            .position(|window| window == &['/' as u8, '/' as u8])
        {
            let comment_size = if let Some(comment_end) = text[comment_start..]
                .windows(1)
                .position(|window| window == &['\n' as u8])
            {
                comment_end + 1
            } else {
                text.len() - comment_start
            };
            Some((comment_start, comment_size))
        } else {
            None
        }
    }

    fn remove_c_style_comments(dts: &[u8]) -> Vec<u8> {
        let mut copy_start = 0;
        let mut new_dts: Vec<u8> = vec![];
        loop {
            if let Some((comment_offset, comment_size)) =
                DtsParser::find_c_comment(&dts[copy_start..])
            {
                for u in &dts[copy_start..(copy_start + comment_offset)] {
                    new_dts.push(*u)
                }

                // And update copy_start to the new location after the comment
                copy_start = copy_start + comment_offset + comment_size;
            } else {
                // No (more) comment was found, copy the text to the end
                // TODO: Copy from copy start ot the end of dts
                for u in &dts[copy_start..] {
                    new_dts.push(*u)
                }
                break;
            }
        }
        new_dts
    }

    fn remove_cpp_style_comments(dts: &[u8]) -> Vec<u8> {
        let mut copy_start = 0;
        let mut new_dts: Vec<u8> = vec![];
        loop {
            if let Some((comment_offset, comment_size)) =
                DtsParser::find_cpp_comment(&dts[copy_start..])
            {
                for u in &dts[copy_start..(copy_start + comment_offset)] {
                    new_dts.push(*u)
                }

                // And update copy_start to the new location after the comment
                copy_start = copy_start + comment_offset + comment_size;
            } else {
                // No (more) comment was found, copy the text to the end
                for u in &dts[copy_start..] {
                    new_dts.push(*u)
                }
                break;
            }
        }
        new_dts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dts_parse_0() {
        // Read the DTS text from test data folder
        let dts = std::fs::read("test/dts_0.dts").unwrap();
        let tree = DtsParser::from_bytes(&dts).parse();
        assert_eq!(tree.root.lock().unwrap().properties.len(), 4);
    }

    #[test]
    fn test_dts_parse_1() {
        // Read the DTS text from test data folder
        let dts = std::fs::read("test/dts_2.dts").unwrap();
        let tree = DtsParser::from_bytes(&dts).parse();
        assert_eq!(tree.root.lock().unwrap().sub_nodes.len(), 1);
        let node_cpus = &tree.root.lock().unwrap().sub_nodes[0];
        assert_eq!(node_cpus.lock().unwrap().sub_nodes.len(), 2);
        assert_eq!(node_cpus.lock().unwrap().properties.len(), 2);
        let node_cpu0 = &node_cpus.lock().unwrap().sub_nodes[0];
        assert_eq!(node_cpu0.lock().unwrap().sub_nodes.len(), 0);
        assert_eq!(node_cpu0.lock().unwrap().properties.len(), 4);
        assert_eq!(
            node_cpu0.lock().unwrap().properties[0].lock().unwrap().name,
            "device_type"
        );
    }

    #[test]
    fn test_dts_parse_remove_c_style_comments_0() {
        let text = "abcdefg /*xxxx xxx xxx */ abcdefg";
        let new_text = DtsParser::remove_c_style_comments(text.as_bytes());
        let new_text = String::from_utf8_lossy(&new_text).to_string();
        assert_eq!("abcdefg  abcdefg", &new_text);
    }

    #[test]
    fn test_dts_parse_remove_c_style_comments_1() {
        let text = "abcdefg\n  /*xxxx \n   *xxx xxx */ \nabcdefg /*****/ /**//**//****/abc";
        let new_text = DtsParser::remove_c_style_comments(text.as_bytes());
        let new_text = String::from_utf8_lossy(&new_text).to_string();
        assert_eq!("abcdefg\n   \nabcdefg  abc", &new_text);
    }

    #[test]
    fn test_dts_parse_remove_c_style_comments_2() {
        let text = "/*xxxx \n   *xxx xxx */ \nabcdefg /*****/ abc /**//**//****/";
        let new_text = DtsParser::remove_c_style_comments(text.as_bytes());
        let new_text = String::from_utf8_lossy(&new_text).to_string();
        assert_eq!(" \nabcdefg  abc ", &new_text);
    }

    #[test]
    fn test_dts_parse_remove_cpp_style_comments_0() {
        let text = "abcdefg // abcdefg \n abcdefg";
        let new_text = DtsParser::remove_cpp_style_comments(text.as_bytes());
        let new_text = String::from_utf8_lossy(&new_text).to_string();
        assert_eq!("abcdefg  abcdefg", &new_text);
    }

    #[test]
    fn test_dts_parse_remove_cpp_style_comments_1() {
        let text = "abcdefg // abcdefg \n// abcdefg \n\nabcdefg";
        let new_text = DtsParser::remove_cpp_style_comments(text.as_bytes());
        let new_text = String::from_utf8_lossy(&new_text).to_string();
        assert_eq!("abcdefg \nabcdefg", &new_text);
    }

    #[test]
    fn test_dts_parse_remove_cpp_style_comments_2() {
        let text = "// abcdefg \n// abcdefg \n\nabcdefg\n//\n// abcdefg\n//";
        let new_text = DtsParser::remove_cpp_style_comments(text.as_bytes());
        let new_text = String::from_utf8_lossy(&new_text).to_string();
        assert_eq!("\nabcdefg\n", &new_text);
    }

    #[test]
    fn test_dts_parse_remove_comments_0() {
        let text = "abcdefg // abcdefg \n/*xxxxx*/////\nabc/**/\n";
        let new_text = DtsParser::remove_c_style_comments(text.as_bytes());
        let new_text = DtsParser::remove_cpp_style_comments(&new_text);
        let new_text = String::from_utf8_lossy(&new_text).to_string();
        assert_eq!("abcdefg abc\n", &new_text);
    }

    #[test]
    fn test_dts_parse_reservation() {
        // Read the DTS text from test data folder
        let dts = std::fs::read("test/dts_4.dts").unwrap();
        let tree = DtsParser::from_bytes(&dts).parse();
        assert_eq!(tree.reservations.len(), 5);
        assert_eq!(tree.reservations[0].lock().unwrap().address, 0x0);
        assert_eq!(tree.reservations[0].lock().unwrap().length, 0x100000);
        assert_eq!(tree.reservations[4].lock().unwrap().address, 0x400000);
        assert_eq!(tree.reservations[4].lock().unwrap().length, 0x100000);
    }

    #[test]
    fn test_dts_parse_deletion() {
        // Read the DTS text from test data folder
        let dts = std::fs::read("test/dts_5.dts").unwrap();
        let tree = DtsParser::from_bytes(&dts).parse();

        assert_eq!(tree.root.lock().unwrap().sub_nodes.len(), 1);
        assert_eq!(
            tree.root.lock().unwrap().sub_nodes[0].lock().unwrap().name,
            "node_b"
        );
        assert_eq!(
            tree.root.lock().unwrap().sub_nodes[0]
                .lock()
                .unwrap()
                .properties
                .len(),
            1
        );
        assert_eq!(
            tree.root.lock().unwrap().sub_nodes[0]
                .lock()
                .unwrap()
                .properties[0]
                .lock()
                .unwrap()
                .name,
            "property_key_0"
        );
        assert_eq!(
            tree.root.lock().unwrap().sub_nodes[0]
                .lock()
                .unwrap()
                .properties[0]
                .lock()
                .unwrap()
                .value,
            vec!['v' as u8, '_' as u8, '0' as u8, 0 as u8]
        );
    }

    #[test]
    fn test_dts_parse_pre_process() {
        let dts = std::fs::read_to_string("test/dts_6.dts").unwrap();
        let dts = DtsParser::pre_process(&dts, 8).unwrap_or(dts);
        assert_eq!(dts.find("/include/").is_none(), true);
        assert_eq!(dts.find("#address-cells").is_some(), true);
    }

    #[test]
    fn test_dts_parse_label() {
        let dts = std::fs::read_to_string("test/dts_7.dts").unwrap();
        let tree = DeviceTree::from_dts_bytes(dts.as_bytes());
        assert_eq!(
            tree.root.lock().unwrap().sub_nodes[2]
                .lock()
                .unwrap()
                .label
                .as_ref()
                .unwrap(),
            "interrupt_controller"
        );
        let prop = tree.root.lock().unwrap().find_property("interrupt-parent");
        assert_eq!(prop.is_some(), true);
        let phandle = u32::from_be_bytes(
            prop.unwrap().lock().unwrap().value[0..4]
                .try_into()
                .unwrap(),
        );
        assert_eq!(phandle, 0);
    }
}
