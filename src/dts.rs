// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::{attribute::Attribute, node::Node, tree::Tree};

pub struct Dts {}

pub struct DtsParser {}

impl DtsParser {
    // Remove comments;
    // Handle compiler instructions;
    // Remove DTS header.
    fn parse(dts: &[u8]) -> Tree {
        // Remove comments
        let dts = &DtsParser::remove_c_style_comments(dts);
        let dts = &DtsParser::remove_cpp_style_comments(dts);

        // TODO: Compiler instructions

        let mut root_node = Node::new("/");
        // The remaining content should be root node(s)
        let mut i: usize = 0;
        let mut text: Vec<u8> = vec![];
        while i < dts.len() {
            match dts[i] as char {
                '{' => {
                    // Found node
                    let node_name =
                        &String::from(String::from_utf8_lossy(&text).to_string().trim());
                    // TODO: the node_name must be "/", fail otherwise
                    println!("found node {}", node_name);
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
        Tree::new(root_node)
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
                    let mut sub_node = Node::new(sub_node_name);
                    println!("found node {}", sub_node_name);
                    i = i + 1;
                    let node_size = DtsParser::parse_node(&dts[i..], &mut sub_node);
                    node.add_sub_node(sub_node);
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
                            DtsParser::handle_directive(&attr_name);
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
                u32::from_str_radix(&num[2..], 10).unwrap()
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
                u8::from_str_radix(&num[2..], 10).unwrap()
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

    fn handle_directive(directive: &str) {
        println!("handle directive: {directive}");
        let mut slices = directive.split_ascii_whitespace();
        let instruction = slices.next().unwrap();
        if instruction == "/delete-node/" {
            let node_path = slices.next().unwrap();
            println!("delete node: {node_path}");
        } else if instruction == "/delete-property/" {
            let property_path = slices.next().unwrap();
            println!("delete property: {property_path}");
        } else {
            panic!("unknown comipler directive {directive}")
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
    use crate::attribute::Attribute;
    use crate::node::Node;
    use crate::tree::Tree;

    #[ignore]
    #[test]
    fn test_dts_generate_0() {
        // Read the DTS text from test data folder
        let dts_0_text = std::fs::read_to_string("test/dts_0.dts").unwrap();
        println!("{dts_0_text}");
        // Build the same device tree with API and compare
        let mut root = Node::new("");
        root.add_attr(Attribute::new_strings(
            "compatible",
            vec![String::from("linux,dummy-virt")],
        ));
        root.add_attr(Attribute::new_u32("#address-cells", 2u32));
        root.add_attr(Attribute::new_u32("#size-cells", 2u32));
        root.add_attr(Attribute::new_u32("interrupt-parent", 1u32));
        let dt = Tree::new(root);
        let dts = dt.to_dts(0);
        assert_eq!(dts_0_text, dts);
    }

    #[ignore]
    #[test]
    fn test_dts_generate_1() {
        // Read the DTS text from test data folder
        let dts_1_text = std::fs::read_to_string("test/dts_1.dts").unwrap();
        println!("{dts_1_text}");
        // Build the same device tree with API and compare
        let mut root = Node::new("");
        root.add_attr(Attribute::new_u32("#address-cells", 2u32));
        root.add_attr(Attribute::new_u32("#size-cells", 2u32));
        let mut memory = Node::new("memory");
        memory.add_attr(Attribute::new_string("device_type", String::from("memory")));
        let reg = vec![0u32, 0x40000000u32, 1u32, 0u32];
        memory.add_attr(Attribute::new_u32s("reg", reg));
        root.add_sub_node(memory);
        let dt = Tree::new(root);
        let dts = dt.to_dts(0);
        assert_eq!(dts_1_text, dts);
    }

    #[ignore]
    #[test]
    fn test_dts_generate_2() {
        // Read the DTS text from test data folder
        let dts_2_text = std::fs::read_to_string("test/dts_2.dts").unwrap();
        println!("{dts_2_text}");

        // Build the same device tree with API and compare
        let mut root = Node::new("");
        root.add_attr(Attribute::new_strings(
            "compatible",
            vec![String::from("linux,dummy-virt")],
        ));
        root.add_attr(Attribute::new_u32("#address-cells", 2u32));
        root.add_attr(Attribute::new_u32("#size-cells", 2u32));
        root.add_attr(Attribute::new_u32("interrupt-parent", 1u32));

        // CPUs
        let mut cpus = Node::new("cpus");
        cpus.add_attr(Attribute::new_u32("#address-cells", 2u32));
        cpus.add_attr(Attribute::new_u32("#size-cells", 0u32));

        // CPU@0
        let mut cpu0 = Node::new("cpu@0");
        cpu0.add_attr(Attribute::new_string("device_type", String::from("cpu")));
        cpu0.add_attr(Attribute::new_strings(
            "compatible",
            vec![String::from("arm,arm-v8")],
        ));
        cpu0.add_attr(Attribute::new_string("enable-method", String::from("psci")));
        let reg = vec![0u32, 0u32];
        cpu0.add_attr(Attribute::new_u32s("reg", reg));
        cpus.add_sub_node(cpu0);

        // CPU@1
        let mut cpu1 = Node::new("cpu@1");
        cpu1.add_attr(Attribute::new_string("device_type", String::from("cpu")));
        cpu1.add_attr(Attribute::new_strings(
            "compatible",
            vec![String::from("arm,arm-v8")],
        ));
        cpu1.add_attr(Attribute::new_string("enable-method", String::from("psci")));
        let reg = vec![0u32, 1u32];
        cpu1.add_attr(Attribute::new_u32s("reg", reg));
        cpus.add_sub_node(cpu1);

        root.add_sub_node(cpus);

        let dt = Tree::new(root);
        let dts = dt.to_dts(0);
        println!("{dts}");
        assert_eq!(dts_2_text, dts);
    }

    #[ignore]
    #[test]
    fn test_dts_generate_3() {
        // Read the DTS text from test data folder
        let dts_3_text = std::fs::read_to_string("test/dts_3.dts").unwrap();
        println!("{dts_3_text}");

        // Build the same device tree with API and compare
        let mut root = Node::new("");
        root.add_attr(Attribute::new_strings(
            "compatible",
            vec![String::from("linux,dummy-virt")],
        ));
        root.add_attr(Attribute::new_u32("#address-cells", 2u32));
        root.add_attr(Attribute::new_u32("#size-cells", 2u32));
        root.add_attr(Attribute::new_u32("interrupt-parent", 1u32));

        // PCI
        let mut pci = Node::new("pci");
        pci.add_attr(Attribute::new_strings(
            "compatible",
            vec![String::from("pci-host-ecam-generic")],
        ));
        pci.add_attr(Attribute::new_string("device_type", String::from("pci")));

        pci.add_attr(Attribute::new_u32s(
            "ranges",
            vec![
                0x2000000u32,
                0x0u32,
                0x10000000u32,
                0x0u32,
                0x10000000u32,
                0x0u32,
                0x20000000u32,
                0x3000000u32,
                0x1u32,
                0x40000000u32,
                0x1u32,
                0x40000000u32,
                0xfeu32,
                0xbfff0000u32,
            ],
        ));
        pci.add_attr(Attribute::new_u32s("bus-range", vec![0u32, 0u32]));
        pci.add_attr(Attribute::new_u32("#address-cells", 0x3u32));
        pci.add_attr(Attribute::new_u32("#size-cells", 0x2u32));
        pci.add_attr(Attribute::new_u32s(
            "reg",
            vec![0u32, 0x30000000u32, 0x0u32, 0x10000000u32],
        ));
        pci.add_attr(Attribute::new_u32("#interrupt-cells", 1u32));
        pci.add_attr(Attribute::new_empty("interrupt-map"));
        pci.add_attr(Attribute::new_empty("interrupt-map-mask"));
        pci.add_attr(Attribute::new_empty("dma-coherent"));
        pci.add_attr(Attribute::new_u32("msi-parent", 0x2u32));

        root.add_sub_node(pci);

        let dt = Tree::new(root);
        let dts = dt.to_dts(0);
        println!("{dts}");
        assert_eq!(dts_3_text, dts);
    }

    #[test]
    fn test_dts_parse_0() {
        // Read the DTS text from test data folder
        let dts = std::fs::read("test/dts_0.dts").unwrap();
        DtsParser::parse(&dts);
    }

    #[test]
    fn test_dts_parse_2() {
        // Read the DTS text from test data folder
        let dts = std::fs::read("test/dts_ori.dts").unwrap();
        DtsParser::parse(&dts);
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
}
