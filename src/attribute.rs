// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dts_generator::DtsGenerator;

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
        let mut bytes: Vec<u8> = value.as_bytes().to_vec();
        bytes.push(0);
        Attribute {
            name: String::from(name),
            value: bytes,
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
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = DtsGenerator::generate_attribute(self, 0);
        writeln!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_attribute_none() {
        let attr = Attribute::new_empty("name");
        assert_eq!(attr.value, vec![]);
    }

    #[test]
    fn test_attribute_u32() {
        let attr = Attribute::new_u32("name", 42);
        assert_eq!(attr.value, vec![0u8, 0u8, 0u8, 42u8]);
    }

    #[test]
    fn test_attribute_u64() {
        let attr = Attribute::new_u64("name", 42);
        assert_eq!(attr.value, vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 42u8]);
    }

    #[test]
    fn test_attribute_str() {
        let s = String::from("hello abc");
        let attr = Attribute::new_string("name", s);
        assert_eq!(
            attr.value,
            vec![
                'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, ' ' as u8, 'a' as u8,
                'b' as u8, 'c' as u8, 0
            ]
        );
    }

    #[test]
    fn test_attribute_strs() {
        let string1 = String::from("hello");
        let string2 = String::from("abc");
        let strs = vec![string1, string2];
        let attr = Attribute::new_strings("name", strs);
        assert_eq!(
            attr.value,
            vec![
                'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 'a' as u8, 'b' as u8,
                'c' as u8, 0
            ]
        );
    }

    #[test]
    fn test_attribute_u8s() {
        let bytes = vec![0u8, 1u8, 2u8, 3u8];
        let attr = Attribute::new_u8s("name", bytes);
        assert_eq!(attr.value, vec![0u8, 1u8, 2u8, 3u8]);
    }

    #[test]
    fn test_attribute_u32s() {
        let bytes = vec![0u32, 1u32, 2u32, 3u32];
        let attr = Attribute::new_u32s("name", bytes);
        assert_eq!(
            attr.value,
            vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 3u8]
        );
    }

    #[test]
    fn test_attribute_print() {
        let attr = Attribute::new_u32("name", 42);
        let printing = format!("{}", attr);
        assert_eq!(&printing, "name = <0x0 0x0 0x0 0x2a>;\n");
    }
}
