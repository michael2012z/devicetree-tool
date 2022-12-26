// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::{attribute::Attribute, node::Node};

pub struct Dts {}

pub struct DtsParser {}

impl DtsParser {
    // Handle compiler instructions;
    // Remove comments;
    // Remove DTS header.
    fn parse(dts: &[u8]) {
        // TODO: Compiler instructions

        let dts = &DtsParser::remove_c_style_comments(dts);
        let dts = &DtsParser::remove_cpp_style_comments(dts);

        // The first item must be the DTS version
        let mut word: Vec<u8> = vec![];
        let mut dts_version: Option<String> = None;
        let mut root: Option<Node> = None;
        let mut block_level = 0;
        let mut i = 0;
        let mut block_start = 0;
        let mut block_end = 0;

        while i < dts.len() {
            match dts[i] as char {
                ';' => {
                    if block_level == 0 {
                        if dts_version.is_none() {
                            // The first identified item must be DTS version: "/dts-v1/"
                            let _dts_version = String::from_utf8_lossy(&word).to_string();
                            let _dts_version = String::from(_dts_version.trim());
                            println!("dts version: {}", _dts_version);
                            dts_version = Some(_dts_version);
                            // TODO: Panic if it is not "/dts-v1/"
                        } else {
                            if root.is_none() {
                                // It must be a node
                                let node_name = String::from_utf8_lossy(&word).to_string();
                                let node_name = String::from(node_name.trim());
                                println!("root node: {}", node_name);
                                root = Some(DtsParser::parse_node(
                                    node_name,
                                    &dts[(block_start + 1)..block_end],
                                ));
                            } else {
                                panic!("multiple root nodes");
                            }
                        }
                        word.clear();
                    }
                }
                '{' => {
                    if block_level == 0 {
                        block_start = i;
                    }
                    block_level = block_level + 1;
                }
                '}' => {
                    block_level = block_level - 1;
                    if block_level == 0 {
                        block_end = i;
                    }
                }
                _ => {
                    if block_level == 0 {
                        word.push(dts[i]);
                    }
                }
            }
            i = i + 1;
        }
        let tail = String::from_utf8_lossy(&word).to_string();
        if tail.trim().len() != 0 {
            panic!("Format error: unfinished content: {}", tail);
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

    fn parse_node(name: String, dts: &[u8]) -> Node {
        let mut word: Vec<u8> = vec![];
        let mut node = Node::new(&name);
        let mut block_level = 0;
        let mut i = 0;
        let mut block_start = 0;
        let mut block_end = 0;
        let mut is_block = false;

        while i < dts.len() {
            match dts[i] as char {
                ';' => {
                    if block_level == 0 {
                        if is_block {
                            let node_name = String::from_utf8_lossy(&word).to_string();
                            let node_name = String::from(node_name.trim());
                            println!("sub_node: {}", node_name);
                            let sub_node = DtsParser::parse_node(
                                node_name,
                                &dts[(block_start + 1)..block_end],
                            );
                            node.add_sub_node(sub_node);
                        } else {
                            let attr = DtsParser::parse_attribute(&word);
                            node.add_attr(attr);
                        }
                        word.clear();
                    }
                }
                '{' => {
                    if block_level == 0 {
                        block_start = i;
                        is_block = true;
                    }
                    block_level = block_level + 1;
                }
                '}' => {
                    block_level = block_level - 1;
                    if block_level == 0 {
                        block_end = i;
                    }
                }
                _ => {
                    if block_level == 0 {
                        word.push(dts[i]);
                    }
                }
            }
            i = i + 1;
        }
        let tail = String::from_utf8_lossy(&word).to_string();
        if tail.trim().len() != 0 {
            panic!("Format error: unfinished content: {}", tail);
        }

        node
    }

    fn parse_attribute(dts: &[u8]) -> Attribute {
        let text = String::from_utf8_lossy(&dts).to_string();
        let text = String::from(text.trim());
        if let Some(eq) = text.find("=") {
            let key = text[0..(eq - 1)].trim();
            let value = text[(eq + 1)..].trim();

            let value_bytes = value.as_bytes();
            if value_bytes.len() < 2 {
                panic!("Attribute value format error: {value}")
            }

            let first_char = value_bytes[0] as char;
            let last_char = value_bytes[value_bytes.len() - 1] as char;

            if first_char == '"' && last_char == '"' {
                let mut strings: Vec<String> = vec![];
                let sp = value.split("\"");

                for (i, s) in sp.enumerate() {
                    if i % 2 == 1 {
                        println!("{s}");
                        strings.push(String::from(s));
                    }
                }
                Attribute::new_strings(key, strings)
            } else if first_char == '<' && last_char == '>' {
                println!(
                    "attribute: key: {key}, value data: {}",
                    &value[1..(value_bytes.len() - 1)]
                );
                let mut u32s: Vec<u32> = vec![];
                for d in value[1..(value_bytes.len() - 1)].split_whitespace() {
                    let u = parse_int::parse::<u32>(d).unwrap();
                    u32s.push(u);
                    println!("{u}");
                }
                Attribute::new_u32s(key, u32s)
            } else {
                panic!("Attribute value format error: {value}")
            }
        } else {
            println!("attribute: key: {}", text);
            Attribute::new_empty(&text)
        }
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
