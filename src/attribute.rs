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
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = DtsGenerator::generate_attribute(self, 0);
        writeln!(f, "{s}")
    }
}
