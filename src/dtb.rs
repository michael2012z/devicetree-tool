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
}
