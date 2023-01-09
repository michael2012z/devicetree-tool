// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use crate::{attribute::Attribute, node::Node, reservation::Reservation, tree::Tree};

pub struct DtsParser {}

impl DtsParser {
    // Remove comments;
    // Handle compiler instructions;
    // Remove DTS header.
    pub fn parse(dts: &[u8]) -> Tree {
        // Remove comments
        let dts = &DtsParser::remove_c_style_comments(dts);
        let dts = &DtsParser::remove_cpp_style_comments(dts);

        // TODO: Compiler instructions

        let mut root_node = Node::new("/");
        let mut reservations: Vec<Rc<Reservation>> = vec![];
        // The remaining content should be root node(s)
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
                        reservations.push(Rc::new(Reservation::new(address, length)));
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
                    let node_size = DtsParser::parse_node(&dts[i..], &mut root_node);
                    i = i + node_size;
                    text.clear();
                }
                _ => {
                    text.push(dts[i]);
                    i = i + 1;
                }
            }
        }
        Tree::new(reservations, root_node)
    }

    fn parse_node(dts: &[u8], node: &mut Node) -> usize {
        let mut i: usize = 0;
        let mut text: Vec<u8> = vec![];
        let mut at_end = false;
        while i < dts.len() {
            match dts[i] as char {
                '{' => {
                    // Found node
                    let sub_node_name =
                        &String::from(String::from_utf8_lossy(&text).to_string().trim());
                    println!("found node {}", sub_node_name);

                    // If a sub_node with the name doesn't exist, create one
                    if !node.sub_nodes.iter().any(|x| &x.name == sub_node_name) {
                        let new_sub_node = Node::new(sub_node_name);
                        node.add_sub_node(new_sub_node);
                    }

                    // Get the sub_node out and update
                    let sub_node = node
                        .sub_nodes
                        .iter_mut()
                        .find(|x| &x.name == sub_node_name)
                        .unwrap();

                    i = i + 1;
                    let node_size = DtsParser::parse_node(&dts[i..], sub_node);
                    i = i + node_size;
                    text.clear();
                }
                '}' => {
                    // Come to the end of current node, expecting a ';' to finish
                    at_end = true;
                    i = i + 1;
                }
                '=' => {
                    // Found an attribute with value
                    let attr_name =
                        &String::from(String::from_utf8_lossy(&text).to_string().trim());
                    println!("found attribute {} with value:", attr_name);
                    i = i + 1;
                    let (attribute_value_size, attribute_value) =
                        DtsParser::parse_attribute_value(&dts[i..]);
                    let attr = Attribute::new_u8s(attr_name, attribute_value);
                    node.add_attr(attr);
                    i = i + attribute_value_size;
                    text.clear();
                }
                ';' => {
                    // We found either:
                    //  - An attribute without value or a compiler directive
                    //  - A directive like `/delete-node/` or `/delete-property/`
                    //  - Or the end of the node
                    i = i + 1;
                    if at_end {
                        return i;
                    } else {
                        // An attribute without value or a comipler directive
                        let attr_name =
                            String::from(String::from_utf8_lossy(&text).to_string().trim());
                        text.clear();
                        if attr_name.starts_with("/") {
                            // A compiler directive
                            let directive = attr_name;
                            println!("found directive: {directive}");
                            let mut slices = directive.split_ascii_whitespace();
                            let instruction = slices.next().unwrap();
                            if instruction == "/delete-node/" {
                                let sub_node_name = slices.next().unwrap();
                                println!("delete node: {sub_node_name}");
                                let sub_node_index = node
                                    .sub_nodes
                                    .iter()
                                    .position(|x| x.name == sub_node_name)
                                    .unwrap();
                                node.sub_nodes.remove(sub_node_index);
                            } else if instruction == "/delete-property/" {
                                let property_name = slices.next().unwrap();
                                println!("delete property: {property_name}");
                                let property_index = node
                                    .attributes
                                    .iter()
                                    .position(|x| x.name == property_name)
                                    .unwrap();
                                node.attributes.remove(property_index);
                            } else {
                                panic!("unknown comipler directive {directive}")
                            }
                        } else {
                            println!("found attribute {} without value", attr_name);
                            let attr = Attribute::new_empty(&attr_name);
                            node.add_attr(attr);
                        }
                    }
                }
                _ => {
                    text.push(dts[i]);
                    i = i + 1;
                }
            }
        }
        panic!("attribute not ended");
    }

    fn parse_attribute_value(dts: &[u8]) -> (usize, Vec<u8>) {
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
                        panic!("found cell-start while parsing another attribute type {value_type}")
                    }
                    value_type = 1;
                    text.clear();
                }
                '>' => {
                    if value_type != 1 {
                        panic!("found cell-end while parsing another attribute type {value_type}")
                    }
                    value_type = 0;

                    let cells_value = DtsParser::parse_attribute_value_cells(&text);
                    for d in cells_value {
                        value.push(d)
                    }
                    text.clear();
                }
                '[' => {
                    // Bytes type
                    if value_type != 0 {
                        panic!(
                            "found bytes-start while parsing another attribute type {value_type}"
                        )
                    }
                    value_type = 2;
                    text.clear();
                }
                ']' => {
                    if value_type != 2 {
                        panic!("found bytes-end while parsing another attribute type {value_type}")
                    }
                    value_type = 0;

                    let bytes_value = DtsParser::parse_attribute_value_bytes(&text);
                    for d in bytes_value {
                        value.push(d)
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
                        let string_value = DtsParser::parse_attribute_value_string(&text);
                        for d in string_value {
                            value.push(d)
                        }
                        text.clear();
                    } else {
                        panic!("found string while parsing another attribute type {value_type}")
                    }
                }
                '\\' => {
                    // Met an escape char, push the esc char and the next char to buffer
                    text.push(dts[i]);
                    i = i + 1;
                    text.push(dts[i]);
                }
                ';' => {
                    // Conclude the attribute
                    // This is the only exit of the function
                    return (i + 1, value);
                }
                _ => {
                    text.push(dts[i]);
                }
            }
            i = i + 1;
        }
        panic!("attribute not ended");
    }

    fn parse_attribute_value_cells(text: &[u8]) -> Vec<u8> {
        let mut value: Vec<u8> = vec![];
        println!("cells: {}", String::from_utf8_lossy(text));
        println!("cells:");
        for num in String::from_utf8_lossy(text).split_whitespace() {
            let n = if num.starts_with("0x") {
                u32::from_str_radix(&num[2..], 16).unwrap()
            } else {
                u32::from_str_radix(num, 10).unwrap()
            };
            let n_u8_vec = n.to_be_bytes().to_vec();
            for n in n_u8_vec {
                value.push(n);
            }
            println!("{:x}", n);
        }
        value
    }

    fn parse_attribute_value_bytes(text: &[u8]) -> Vec<u8> {
        let mut value: Vec<u8> = vec![];
        println!("bytes: {}", String::from_utf8_lossy(text));
        println!("bytes:");
        for num in String::from_utf8_lossy(text).split_whitespace() {
            let n = if num.starts_with("0x") {
                u8::from_str_radix(&num[2..], 16).unwrap()
            } else {
                u8::from_str_radix(num, 10).unwrap()
            };
            value.push(n);
            println!("{:x}", n);
        }
        value
    }

    fn parse_attribute_value_string(text: &[u8]) -> Vec<u8> {
        println!("string: {}", String::from_utf8_lossy(text));
        text.to_vec()
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
        let tree = DtsParser::parse(&dts);
        assert_eq!(tree.root.attributes.len(), 4);
    }

    #[test]
    fn test_dts_parse_1() {
        // Read the DTS text from test data folder
        let dts = std::fs::read("test/dts_2.dts").unwrap();
        let tree = DtsParser::parse(&dts);
        assert_eq!(tree.root.sub_nodes.len(), 1);
        let node_cpus = &tree.root.sub_nodes[0];
        assert_eq!(node_cpus.sub_nodes.len(), 2);
        assert_eq!(node_cpus.attributes.len(), 2);
        let node_cpu0 = &node_cpus.sub_nodes[0];
        assert_eq!(node_cpu0.sub_nodes.len(), 0);
        assert_eq!(node_cpu0.attributes.len(), 4);
        assert_eq!(node_cpu0.attributes[0].name, "device_type");
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
        let tree = DtsParser::parse(&dts);
        assert_eq!(tree.reservations.len(), 5);
        assert_eq!(tree.reservations[0].address, 0x0);
        assert_eq!(tree.reservations[0].length, 0x100000);
        assert_eq!(tree.reservations[4].address, 0x400000);
        assert_eq!(tree.reservations[4].length, 0x100000);
    }

    #[test]
    fn test_dts_parse_deletion() {
        // Read the DTS text from test data folder
        let dts = std::fs::read("test/dts_5.dts").unwrap();
        let tree = DtsParser::parse(&dts);

        assert_eq!(tree.root.sub_nodes.len(), 1);
        assert_eq!(tree.root.sub_nodes[0].name, "node_b");
        assert_eq!(tree.root.sub_nodes[0].attributes.len(), 1);
        assert_eq!(tree.root.sub_nodes[0].attributes[0].name, "property_key_0");
        assert_eq!(
            tree.root.sub_nodes[0].attributes[0].value,
            vec!['v' as u8, '_' as u8, '0' as u8]
        );
    }
}
