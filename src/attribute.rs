// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::utils::Utils;

pub struct Attribute {
    pub name: String,
    pub value: Vec<u8>,
}

impl Attribute {
    pub fn new_empty(name: &str) -> Self {
        Attribute {
            name: String::from(name),
            value: vec![],
        }
    }
    pub fn new_u32(name: &str, value: u32) -> Self {
        Attribute {
            name: String::from(name),
            value: value.to_be_bytes().to_vec(),
        }
    }
    pub fn new_u64(name: &str, value: u64) -> Self {
        Attribute {
            name: String::from(name),
            value: value.to_be_bytes().to_vec(),
        }
    }
    pub fn new_string(name: &str, value: String) -> Self {
        Attribute {
            name: String::from(name),
            value: value.as_bytes().to_vec(),
        }
    }
    pub fn new_strings(name: &str, value: Vec<String>) -> Self {
        let mut bytes: Vec<u8> = vec![];
        for v in value {
            let mut v_bytes = v.as_bytes().to_vec();
            bytes.append(&mut v_bytes);
            bytes.push(0)
        }
        Attribute {
            name: String::from(name),
            value: bytes,
        }
    }
    pub fn new_u8s(name: &str, value: Vec<u8>) -> Self {
        Attribute {
            name: String::from(name),
            value,
        }
    }
    pub fn new_u32s(name: &str, value: Vec<u32>) -> Self {
        let mut bytes: Vec<u8> = vec![];
        for v in value {
            let mut v_bytes = v.to_be_bytes().to_vec();
            bytes.append(&mut v_bytes)
        }
        Attribute {
            name: String::from(name),
            value: bytes,
        }
    }

    pub fn to_dts(&self, indent_level: u32) -> String {
        let mut s = String::from(format!("{}{} = <", Utils::indent(indent_level), self.name));
        for i in 0..self.value.len() {
            let d = self.value[i];
            if i > 0 {
                s.push(' ')
            }
            s.push_str(&format!("{:#x}", d));
        }
        s.push_str(">;");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_none() {
        let attr = Attribute::new_empty("name");
        println!("{}", attr.to_dts(0));
    }

    #[test]
    fn test_attribute_u32() {
        let attr = Attribute::new_u32("name", 42);
        println!("{}", attr.to_dts(0));
    }

    #[test]
    fn test_attribute_strs() {
        let string1 = String::from("&str abc");
        let string2 = String::from("def");
        let strs = vec![string1, string2];
        let attr = Attribute::new_strings("name", strs);
        println!("{}", attr.to_dts(0));
    }

    #[test]
    fn test_attribute_str() {
        let s = String::from("&str abc");
        let attr = Attribute::new_string("name", s);
        println!("{}", attr.to_dts(0));
    }

    #[test]
    fn test_attribute_bytes() {
        let bytes = vec![0u8, 1u8, 2u8, 3u8];
        let attr = Attribute::new_u8s("name", bytes);
        println!("{}", attr.to_dts(0));
    }
}
