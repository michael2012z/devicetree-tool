// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::attribute::Attribute;
use crate::dtb::{DtbHeader, ReserveEntry};
use crate::node::Node;
use crate::tree::Tree;
use std::rc::Rc;

pub struct DtbGenerator {
    header: DtbHeader,
    reserve_entries: Vec<ReserveEntry>,
    strings_block: Vec<u8>,
    structure_block: Vec<u8>,
    tree: Tree,
}

impl DtbGenerator {
    pub fn from_tree(tree: Tree) -> DtbGenerator {
        let header = DtbHeader {
            magic: 0u32,
            total_size: 0u32,
            off_dt_struct: 0u32,
            off_dt_strings: 0u32,
            off_mem_rsvmap: 0u32,
            version: 0u32,
            last_comp_version: 0u32,
            boot_cpuid_phys: 0u32,
            size_dt_strings: 0u32,
            size_dt_struct: 0u32,
        };
        let reserve_entries: Vec<ReserveEntry> = vec![];
        let strings_block: Vec<u8> = vec![];
        let structure_block: Vec<u8> = vec![];
        DtbGenerator {
            header,
            reserve_entries,
            strings_block,
            structure_block,
            tree,
        }
    }

    pub fn generate(&mut self) -> Vec<u8> {
        let mut reservation_block = self.generate_reservation_block();
        let mut structure_block = self.generate_structure_block();
        let mut strings_block = self.generate_strings_block();

        let mut header_magic = 0xd00dfeedu32.to_be_bytes().to_vec();
        let header_total_size =
            40 + reservation_block.len() + structure_block.len() + strings_block.len();
        let mut header_total_size = (header_total_size as u32).to_be_bytes().to_vec();
        let mut header_off_mem_rsvmap = 40u32.to_be_bytes().to_vec();
        let mut header_off_dt_struct = ((40 + reservation_block.len()) as u32)
            .to_be_bytes()
            .to_vec();
        let mut header_size_dt_struct = (structure_block.len() as u32).to_be_bytes().to_vec();
        let mut header_off_dt_strings = ((40 + reservation_block.len() + structure_block.len())
            as u32)
            .to_be_bytes()
            .to_vec();
        let mut header_size_dt_strings = (strings_block.len() as u32).to_be_bytes().to_vec();
        let mut header_version = (17u32).to_be_bytes().to_vec();
        let mut header_last_comp_version = (16u32).to_be_bytes().to_vec();
        let mut header_boot_cpuid_phys = (0u32).to_be_bytes().to_vec();

        let mut bytes: Vec<u8> = vec![];
        // header
        bytes.append(&mut header_magic);
        bytes.append(&mut header_total_size);
        bytes.append(&mut header_off_dt_struct);
        bytes.append(&mut header_off_dt_strings);
        bytes.append(&mut header_off_mem_rsvmap);
        bytes.append(&mut header_version);
        bytes.append(&mut header_last_comp_version);
        bytes.append(&mut header_boot_cpuid_phys);
        bytes.append(&mut header_size_dt_strings);
        bytes.append(&mut header_size_dt_struct);

        // blocks
        bytes.append(&mut reservation_block);
        bytes.append(&mut structure_block);
        bytes.append(&mut strings_block);

        bytes
    }

    fn generate_attribute(&mut self, attr: &Rc<Attribute>) -> Vec<u8> {
        let mut token = 3u32.to_be_bytes().to_vec();
        let mut len = (attr.value.len() as u32).to_be_bytes().to_vec();
        let name = attr.name.clone();
        let mut nameoff = self.add_string(&name).to_be_bytes().to_vec();

        let mut bytes: Vec<u8> = vec![];
        bytes.append(&mut token);
        bytes.append(&mut len);
        bytes.append(&mut nameoff);
        for d in &attr.value {
            bytes.push(d.to_owned())
        }

        let paddings = ((bytes.len() + 3) >> 2 << 2) - bytes.len();
        for _ in 0..paddings {
            bytes.push(0u8);
        }

        bytes
    }

    fn generate_node(&mut self, node: &Rc<Node>) -> Vec<u8> {
        let mut token = 1u32.to_be_bytes().to_vec();
        let mut name = node.name.clone().as_bytes().to_owned();
        name.push(0u8);

        let mut bytes: Vec<u8> = vec![];

        bytes.append(&mut token);

        bytes.append(&mut name);
        let paddings = ((bytes.len() + 3) >> 2 << 2) - bytes.len();
        for _ in 0..paddings {
            bytes.push(0u8);
        }

        for attr in node.attributes.iter() {
            let mut attr_bytes = self.generate_attribute(attr);
            bytes.append(&mut attr_bytes);
        }

        for sub_node in node.sub_nodes.iter() {
            let mut node_bytes = self.generate_node(sub_node);
            bytes.append(&mut node_bytes);
        }

        let mut token = 2u32.to_be_bytes().to_vec();
        bytes.append(&mut token);

        bytes
    }

    fn generate_structure_block(&mut self) -> Vec<u8> {
        let root = &self.tree.root.clone();
        let mut token = 9u32.to_be_bytes().to_vec();

        let mut bytes = self.generate_node(root);
        bytes.append(&mut token);

        bytes
    }

    fn generate_reservation_block(&self) -> Vec<u8> {
        let mut address = 0u64.to_be_bytes().to_vec();
        let mut size = 0u64.to_be_bytes().to_vec();

        let mut bytes: Vec<u8> = vec![];
        bytes.append(&mut address);
        bytes.append(&mut size);

        bytes
    }

    fn add_string(&mut self, s: &str) -> u32 {
        let len = self.strings_block.len() as u32;

        for c in s.bytes() {
            self.strings_block.push(c)
        }
        self.strings_block.push(0u8);

        len
    }

    fn generate_strings_block(&mut self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];

        for c in &self.strings_block {
            bytes.push(c.to_owned());
        }

        let paddings = ((bytes.len() + 3) >> 2 << 2) - bytes.len();
        for _ in 0..paddings {
            bytes.push(0u8);
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dtb_parser::DtbParser;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_dtb_generate_0() {
        // Build a simple device tree
        let mut root = Node::new("");
        root.add_attr(Attribute::new_strings(
            "compatible",
            vec![String::from("linux,dummy-virt")],
        ));
        let tree = Tree::new(root);

        let mut dtb_generator = DtbGenerator::from_tree(tree);
        let dtb_bytes = dtb_generator.generate();

        let tree = DtbParser::from_bytes(&dtb_bytes).parse();
        let s = tree.to_dts(0);
        println!("{}\n{}", s.len(), s);
        assert_eq!(tree.root.name, "");
        assert_eq!(tree.root.attributes[0].name, "compatible");
        assert_eq!(tree.root.attributes[0].value.len(), 17);
    }

    #[test]
    fn test_dtb_generate_1() {
        // Build a simple device tree
        let mut root = Node::new("");
        root.add_attr(Attribute::new_strings(
            "compatible",
            vec![String::from("linux,dummy-virt")],
        ));
        root.add_attr(Attribute::new_u32("#address-cells", 2u32));
        root.add_attr(Attribute::new_u32("#size-cells", 2u32));
        root.add_attr(Attribute::new_u32("interrupt-parent", 1u32));
        let tree = Tree::new(root);
        let s = tree.to_dts(0);
        println!("{}\n{}", s.len(), s);
        assert_eq!(tree.root.attributes[0].name, "compatible");
        assert_eq!(tree.root.attributes[0].value.len(), 17);
        assert_eq!(tree.root.attributes[1].name, "#address-cells");
        assert_eq!(
            u32::from_be_bytes(tree.root.attributes[1].value[0..4].try_into().unwrap()),
            2u32
        );
        assert_eq!(tree.root.attributes[2].name, "#size-cells");
        assert_eq!(
            u32::from_be_bytes(tree.root.attributes[2].value[0..4].try_into().unwrap()),
            2u32
        );
        assert_eq!(tree.root.attributes[3].name, "interrupt-parent");
        assert_eq!(
            u32::from_be_bytes(tree.root.attributes[3].value[0..4].try_into().unwrap()),
            1u32
        );
    }

    #[test]
    fn test_dtb_parse_0() {
        let mut f = File::open("test/dtb_0.dtb").unwrap();
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer).unwrap();

        let tree = DtbParser::from_bytes(&buffer).parse();
        let tree_string = tree.to_dts(0);
        println!("{}\n{}", tree_string.len(), tree_string);

        // find the number of "="
        let mut str = tree_string.as_str();
        let mut count = 0;
        loop {
            if let Some(index) = str.find("=") {
                count = count + 1;
                str = &str[(index + 1)..];
            } else {
                break;
            }
        }
        assert_eq!(count, 84);

        // find the number of "};"
        let mut str = tree_string.as_str();
        let mut count = 0;
        loop {
            if let Some(index) = str.find("};") {
                count = count + 1;
                str = &str[(index + 2)..];
            } else {
                break;
            }
        }
        assert_eq!(count, 19);
    }

    #[test]
    fn test_dtb_generate_9() {
        let mut f = File::open("test/dtb_0.dtb").unwrap();
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer).unwrap();

        let tree = DtbParser::from_bytes(&buffer).parse();

        let mut dtb_generator = DtbGenerator::from_tree(tree);
        let dtb_bytes = dtb_generator.generate();

        // parse the generated DTB
        let tree = DtbParser::from_bytes(&dtb_bytes).parse();
        let tree_string = tree.to_dts(0);
        println!("{}\n{}", tree_string.len(), tree_string);

        // find the number of "="
        let mut str = tree_string.as_str();
        let mut count = 0;
        loop {
            if let Some(index) = str.find("=") {
                count = count + 1;
                str = &str[(index + 1)..];
            } else {
                break;
            }
        }
        assert_eq!(count, 84);

        // find the number of "};"
        let mut str = tree_string.as_str();
        let mut count = 0;
        loop {
            if let Some(index) = str.find("};") {
                count = count + 1;
                str = &str[(index + 2)..];
            } else {
                break;
            }
        }
        assert_eq!(count, 19);
    }
}
