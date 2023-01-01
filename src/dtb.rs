// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

pub struct DtbHeader {
    pub magic: u32,
    pub total_size: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}

pub struct ReserveEntry {
    pub address: u64,
    pub size: u64,
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
