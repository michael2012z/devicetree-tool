// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::attribute::Attribute;
use crate::node::Node;
use crate::tree::Tree;
use std::rc::Rc;

pub struct Dtb {}

struct DtbParser {
    header: DtbHeader,
    reserve_entries: Vec<ReserveEntry>,
    strings_block: Vec<u8>,
    structure_block: Vec<u8>,
}

struct DtbGenerator {
    header: DtbHeader,
    reserve_entries: Vec<ReserveEntry>,
    strings_block: Vec<u8>,
    structure_block: Vec<u8>,
    tree: Tree,
}

struct DtbHeader {
    magic: u32,
    total_size: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

struct ReserveEntry {
    address: u64,
    size: u64,
}

impl std::fmt::Display for DtbHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "magic: 0x{:x}", self.magic)?;
        writeln!(f, "total_size: 0x{:x}", self.total_size)?;
        writeln!(f, "off_dt_struct: 0x{:x}", self.off_dt_struct)?;
        writeln!(f, "off_dt_strings: 0x{:x}", self.off_dt_strings)?;
        writeln!(f, "off_mem_rsvmap: 0x{:x}", self.off_mem_rsvmap)?;
        writeln!(f, "version: {}", self.version)?;
        writeln!(f, "last_comp_version: {}", self.last_comp_version)?;
        writeln!(f, "boot_cpuid_phys: 0x{:x}", self.boot_cpuid_phys)?;
        writeln!(f, "size_dt_strings: 0x{:x}", self.size_dt_strings)?;
        writeln!(f, "size_dt_struct: 0x{:x}", self.size_dt_struct)
    }
}

impl Dtb {
    pub fn parse_dtb_bytes(bytes: &[u8]) -> Tree {
        DtbParser::from_bytes(bytes).parse()
    }

    pub fn generate_dtb_bytes(tree: Tree) -> Vec<u8> {
        let mut dtb_generator = DtbGenerator::from_tree(tree);
        dtb_generator.generate()
    }
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

    #[test]
    fn test_dtb_generate_0() {
        // Build a simple device tree
        let mut root = Node::new("");
        root.add_attr(Attribute::new_strings(
            "compatible",
            vec![String::from("linux,dummy-virt")],
        ));
        let tree = Tree::new(root);
        let dtb_bytes = Dtb::generate_dtb_bytes(tree);
        let tree = Dtb::parse_dtb_bytes(&dtb_bytes);
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

        let tree = Dtb::parse_dtb_bytes(&buffer);
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

        let tree = Dtb::parse_dtb_bytes(&buffer);
        let dtb_bytes = Dtb::generate_dtb_bytes(tree);

        // parse the generated DTB
        let tree = Dtb::parse_dtb_bytes(&dtb_bytes);
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
