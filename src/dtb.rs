// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

pub struct Dtb {}

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
            v.push(ReserveEntry { address, size })
        }
        v
    }

    fn get_string(strings_block: &[u8], offset: u32) -> String {
        let mut s = String::new();
        for i in (offset as usize)..strings_block.len() {
            if strings_block[i] != 0 {
                s.push(strings_block[i] as char)
            } else {
                break;
            }
        }
        s
    }

    // struct_block starts immediately after the FDT_BEGIN_NODE token,
    // in the beginning it should be the node name.
    // The end of struct_block should be a FDT_END_NODE token
    // Return the position next to the FDT_END_NODE token. That is also the length
    // from the node name to the end of the node.
    fn parse_structure_node(struct_block: &[u8]) -> usize {
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

        while pos < struct_block.len() {
            let token = u32::from_be_bytes(struct_block[pos..(pos + 4)].try_into().unwrap());
            pos = pos + 4;
            match token {
                0 => {
                    println!("zeroed pedding at 0x{:x}", pos - 4);
                }
                1 => {
                    println!("FDT_BEGIN_NODE at 0x{:x}", pos - 4);
                    let node_len = Dtb::parse_structure_node(&struct_block[pos..]);
                    pos = pos + node_len;
                }
                2 => {
                    println!("FDT_END_NODE at 0x{:x}", pos - 4);
                    return pos;
                }
                3 => {
                    println!("FDT_PROP at 0x{:x}", pos - 4);
                    let prop_len = Dtb::parse_structure_prop(&struct_block[pos..]);
                    pos = pos + prop_len;
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

    fn parse_structure_prop(struct_block: &[u8]) -> usize {
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
        pos
    }

    fn parse_structure_block(struct_block: &[u8]) {
        //Dtb::print_structure_block(struct_block);
        let token = u32::from_be_bytes(struct_block[0..4].try_into().unwrap());
        // The first token must be the root node of the tree
        if token != 1 {
            panic!("Root node is not found")
        }

        let next_pos = 4 + Dtb::parse_structure_node(&struct_block[4..]);
        let token = u32::from_be_bytes(struct_block[next_pos..(next_pos + 4)].try_into().unwrap());
        // The FDT_END token should follow the root node immediately
        if token != 9 {
            panic!("Root node is not found")
        }
    }

    fn print_structure_block(struct_block: &[u8]) {
        let mut pos = 0usize;
        while pos < struct_block.len() {
            let token = u32::from_be_bytes(struct_block[pos..(pos + 4)].try_into().unwrap());
            pos = pos + 4;
            match token {
                0 => {
                    println!("zeroed pedding at 0x{:x}", pos - 4);
                }
                1 => {
                    println!("FDT_BEGIN_NODE at 0x{:x}", pos - 4);
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
                    println!("Node name: {}, next pos = 0x{:x}", name, pos)
                }
                2 => {
                    println!("FDT_END_NODE at 0x{:x}", pos - 4);
                }
                3 => {
                    println!("FDT_PROP at 0x{:x}", pos - 4);
                    let prop_len =
                        u32::from_be_bytes(struct_block[pos..(pos + 4)].try_into().unwrap());
                    pos = pos + 4;
                    let prop_nameoff =
                        u32::from_be_bytes(struct_block[pos..(pos + 4)].try_into().unwrap());
                    pos = pos + 4;
                    let prop_data = &struct_block[pos..(pos + prop_len as usize)];
                    pos = pos + prop_len as usize;
                    pos = (pos + 3) >> 2 << 2;
                    println!(
                        "Property: name_pos 0x{:x}, len 0x{:x}, data {:#?}, next pos 0x{:x}",
                        prop_nameoff, prop_len, prop_data, pos
                    );
                }
                4 => {
                    println!("FDT_NOP at 0x{:x}", pos - 4);
                }
                9 => {
                    println!("FDT_END at 0x{:x}", pos - 4);
                }
                _ => {
                    panic!("unknow token 0x{:x} at 0x{:x}", token, pos - 4)
                }
            }
        }
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
        let header = Dtb::parse_header(&buffer[0..40]);
        println!("header: \n{:#}", header);
        assert_eq!(0xd00dfeed, header.magic);
        assert_eq!(17, header.version);
        assert_eq!(16, header.last_comp_version);
    }

    #[test]
    fn test_dtb_parse_strings() {
        let mut f = File::open("test/dtb_0.dtb").unwrap();
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer).unwrap();

        // parse the header
        let header = Dtb::parse_header(&buffer[0..40]);

        // get the strings block
        let strings_block = &buffer[(header.off_dt_strings as usize)
            ..(header.off_dt_strings + header.size_dt_strings) as usize];

        assert_eq!(Dtb::get_string(strings_block, 0), "compatible");
        assert_eq!(Dtb::get_string(strings_block, 11), "#address-cells");
        assert_eq!(Dtb::get_string(strings_block, 38), "interrupt-parent");
        assert_eq!(Dtb::get_string(strings_block, 94), "interrupt-controller");
        assert_eq!(Dtb::get_string(strings_block, 147), "interrupts");
    }

    #[test]
    fn test_dtb_reservation_block() {
        let mut f = File::open("test/dtb_0.dtb").unwrap();
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer).unwrap();

        // parse the header
        let header = Dtb::parse_header(&buffer[0..40]);

        // parse the reservation block
        let mut addr = header.off_mem_rsvmap as usize;

        while addr < header.total_size as usize {
            let address = u64::from_be_bytes(buffer[addr..(addr + 8)].try_into().unwrap());
            let size = u64::from_be_bytes(buffer[(addr + 8)..(addr + 16)].try_into().unwrap());

            println!("ReserveEntry: addr 0x{:x}, size 0x{:x}", address, size);
            if address == 0 && size == 0 {
                break;
            }
            addr = addr + 16;
        }
        let reservation_block = &buffer[(header.off_mem_rsvmap as usize)..(addr)];

        let v = Dtb::parse_reservation_block(reservation_block);
        for r in v {
            println!("ReserveEntry: addr 0x{:x}, size 0x{:x}", r.address, r.size);
        }
    }

    #[test]
    fn test_dtb_parse_structure_block() {
        let mut f = File::open("test/dtb_0.dtb").unwrap();
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer).unwrap();

        // parse the header
        let header = Dtb::parse_header(&buffer[0..40]);

        // get the struct block
        let struct_block = &buffer[(header.off_dt_struct as usize)
            ..(header.off_dt_struct + header.size_dt_struct) as usize];

        Dtb::parse_structure_block(struct_block);
    }
}
