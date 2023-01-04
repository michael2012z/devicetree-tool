// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::attribute::Attribute;
use crate::dtb::{DtbHeader, ReserveEntry};
use crate::node::Node;
use crate::tree::Tree;

#[allow(dead_code)]
pub struct DtbParser {
    header: DtbHeader,
    reserve_entries: Vec<ReserveEntry>,
    strings_block: Vec<u8>,
    structure_block: Vec<u8>,
}

impl DtbParser {
    pub fn from_bytes(bytes: &[u8]) -> DtbParser {
        if bytes.len() < 40 {
            panic!("Invalid header format")
        }
        let header = DtbParser::parse_header(&bytes[0..40]);

        let reservation_block = &bytes[(header.off_mem_rsvmap as usize)..];

        let reserve_entries = DtbParser::parse_reservation_block(reservation_block);

        let strings_block = &bytes[(header.off_dt_strings as usize)
            ..(header.off_dt_strings + header.size_dt_strings) as usize];
        let strings_block = strings_block.to_owned();

        let structure_block = &bytes[(header.off_dt_struct as usize)
            ..(header.off_dt_struct + header.size_dt_struct) as usize];
        let structure_block = structure_block.to_owned();

        DtbParser {
            header,
            reserve_entries,
            strings_block,
            structure_block,
        }
    }

    pub fn parse(&self) -> Tree {
        self.parse_structure_block(self.structure_block.as_ref())
    }

    fn parse_header(header: &[u8]) -> DtbHeader {
        let magic = u32::from_be_bytes(header[0..4].try_into().unwrap());
        let total_size = u32::from_be_bytes(header[4..8].try_into().unwrap());
        let off_dt_struct = u32::from_be_bytes(header[8..12].try_into().unwrap());
        let off_dt_strings = u32::from_be_bytes(header[12..16].try_into().unwrap());
        let off_mem_rsvmap = u32::from_be_bytes(header[16..20].try_into().unwrap());
        let version = u32::from_be_bytes(header[20..24].try_into().unwrap());
        let last_comp_version = u32::from_be_bytes(header[24..28].try_into().unwrap());
        let boot_cpuid_phys = u32::from_be_bytes(header[28..32].try_into().unwrap());
        let size_dt_strings = u32::from_be_bytes(header[32..36].try_into().unwrap());
        let size_dt_struct = u32::from_be_bytes(header[36..40].try_into().unwrap());

        DtbHeader {
            magic,
            total_size,
            off_dt_struct,
            off_dt_strings,
            off_mem_rsvmap,
            version,
            last_comp_version,
            boot_cpuid_phys,
            size_dt_strings,
            size_dt_struct,
        }
    }

    fn get_string(&self, offset: u32) -> String {
        let mut s = String::new();
        for i in (offset as usize)..self.strings_block.len() {
            if self.strings_block[i] != 0 {
                s.push(self.strings_block[i] as char)
            } else {
                break;
            }
        }
        s
    }

    // reservation_block may contain the bytes after the actual reservation block.
    // The real reservation block is zero-terminated.
    fn parse_reservation_block(reservation_block: &[u8]) -> Vec<ReserveEntry> {
        let mut v = Vec::new();
        for i in 0..(reservation_block.len() / 16 as usize) {
            let address = u64::from_be_bytes(
                reservation_block[(i * 16)..(i * 16 + 8)]
                    .try_into()
                    .unwrap(),
            );
            let size = u64::from_be_bytes(
                reservation_block[(i * 16 + 8)..(i * 16 + 16)]
                    .try_into()
                    .unwrap(),
            );
            if address == 0 && size == 0 {
                break;
            } else {
                v.push(ReserveEntry { address, size })
            }
        }
        v
    }

    fn parse_structure_block(&self, structure_block: &[u8]) -> Tree {
        let token = u32::from_be_bytes(structure_block[0..4].try_into().unwrap());
        // The first token must be the root node of the tree
        if token != 1 {
            panic!("Root node is not found")
        }

        let (root_len, root_node) = self.parse_structure_node(&structure_block[4..]);
        let next_pos = 4 + root_len;
        let token = u32::from_be_bytes(
            structure_block[next_pos..(next_pos + 4)]
                .try_into()
                .unwrap(),
        );
        // The FDT_END token should follow the root node immediately
        if token != 9 {
            panic!("Root node is not found")
        }

        Tree::new(root_node)
    }

    // struct_block starts immediately after the FDT_BEGIN_NODE token,
    // in the beginning it should be the node name.
    // The end of struct_block should be a FDT_END_NODE token
    // Return the position next to the FDT_END_NODE token. That is also the length
    // from the node name to the end of the node.
    fn parse_structure_node(&self, struct_block: &[u8]) -> (usize, Node) {
        let mut pos = 0usize;
        // find the node name
        let mut name = String::new();
        while struct_block[pos] != 0 {
            name.push(struct_block[pos] as char);
            pos = pos + 1;
        }
        // move to the next postion after the zero-terminated string
        pos = pos + 1;
        // align to 4-bytes
        pos = (pos + 3) >> 2 << 2;
        println!("Node name: {}, next pos = 0x{:x}", name, pos);
        let mut node = Node::new(&name);

        while pos < struct_block.len() {
            let token = u32::from_be_bytes(struct_block[pos..(pos + 4)].try_into().unwrap());
            pos = pos + 4;
            match token {
                0 => {
                    println!("zeroed pedding at 0x{:x}", pos - 4);
                }
                1 => {
                    println!("FDT_BEGIN_NODE at 0x{:x}", pos - 4);
                    let (node_len, sub_node) = self.parse_structure_node(&struct_block[pos..]);
                    pos = pos + node_len;
                    node.add_sub_node(sub_node);
                }
                2 => {
                    println!("FDT_END_NODE at 0x{:x}", pos - 4);
                    return (pos, node);
                }
                3 => {
                    println!("FDT_PROP at 0x{:x}", pos - 4);
                    let (prop_len, attribute) = self.parse_structure_prop(&struct_block[pos..]);
                    pos = pos + prop_len;
                    node.add_attr(attribute);
                }
                4 => {
                    println!("FDT_NOP at 0x{:x}", pos - 4);
                }
                _ => {
                    panic!("unknow token 0x{:x} at 0x{:x}", token, pos - 4)
                }
            }
        }
        panic!("Node not closed")
    }

    fn parse_structure_prop(&self, struct_block: &[u8]) -> (usize, Attribute) {
        let mut pos = 0usize;
        let prop_len = u32::from_be_bytes(struct_block[pos..(pos + 4)].try_into().unwrap());
        pos = pos + 4;
        let prop_nameoff = u32::from_be_bytes(struct_block[pos..(pos + 4)].try_into().unwrap());
        pos = pos + 4;
        let prop_data = &struct_block[pos..(pos + prop_len as usize)];
        pos = pos + prop_len as usize;
        pos = (pos + 3) >> 2 << 2;
        println!(
            "Property: name_pos 0x{:x}, len 0x{:x}, data {:#?}, next pos 0x{:x}",
            prop_nameoff, prop_len, prop_data, pos
        );
        let attr_name = self.get_string(prop_nameoff);
        let attribute = Attribute::new_u8s(&attr_name, prop_data.to_owned());
        (pos, attribute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_dtb_parse_header() {
        let mut f = File::open("test/dtb_0.dtb").unwrap();
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer).unwrap();
        assert_eq!(2672, buffer.len());

        // parse the header
        let header = DtbParser::parse_header(&buffer[0..40]);
        println!("header: \n{:#}", header);
        assert_eq!(0xd00dfeed, header.magic);
        assert_eq!(17, header.version);
        assert_eq!(16, header.last_comp_version);
    }

    #[test]
    fn test_dtb_parse_strings_block() {
        let mut f = File::open("test/dtb_0.dtb").unwrap();
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer).unwrap();

        let dtb_parser = DtbParser::from_bytes(&buffer);

        assert_eq!(dtb_parser.get_string(0), "compatible");
        assert_eq!(dtb_parser.get_string(11), "#address-cells");
        assert_eq!(dtb_parser.get_string(38), "interrupt-parent");
        assert_eq!(dtb_parser.get_string(94), "interrupt-controller");
        assert_eq!(dtb_parser.get_string(147), "interrupts");
    }

    #[test]
    fn test_dtb_parse_reservation_block() {
        let mut f = File::open("test/dtb_0.dtb").unwrap();
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer).unwrap();

        let dtb_parser = DtbParser::from_bytes(&buffer);

        let v = dtb_parser.reserve_entries;
        assert_eq!(v.len(), 0);
    }
}
