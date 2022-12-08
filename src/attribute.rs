// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::element::Element;

pub struct Attribute<T> {
    name: String,
    value: T,
}

impl<T> Attribute<T> {
    pub fn new(name: &str, value: T) -> Self {
        Attribute {
            name: String::from(name),
            value,
        }
    }
}

impl Element for Attribute<Option<u32>> {
    fn to_dts(&self, indent_level: u32) -> String {
        let mut indents = String::new();
        for i in 0..indent_level {
            indents.push('\t')
        }
        String::from(format!("{indents}{};", self.name))
    }
}

impl Element for Attribute<u32> {
    fn to_dts(&self, indent_level: u32) -> String {
        let mut indents = String::new();
        for i in 0..indent_level {
            indents.push('\t')
        }
        String::from(format!("{indents}{} = <{:#x}>;", self.name, self.value))
    }
}

impl Element for Attribute<String> {
    fn to_dts(&self, indent_level: u32) -> String {
        let mut indents = String::new();
        for i in 0..indent_level {
            indents.push('\t')
        }
        String::from(format!("{indents}{} = \"{}\";", self.name, self.value))
    }
}

impl Element for Attribute<Vec<u8>> {
    fn to_dts(&self, indent_level: u32) -> String {
        let mut indents = String::new();
        for i in 0..indent_level {
            indents.push('\t')
        }
        let mut s = String::from(format!("{indents}{} = <", self.name));
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

impl Element for Attribute<Vec<u32>> {
    fn to_dts(&self, indent_level: u32) -> String {
        let mut indents = String::new();
        for i in 0..indent_level {
            indents.push('\t')
        }
        let mut s = String::from(format!("{indents}{} = <", self.name));
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

impl Element for Attribute<Vec<String>> {
    fn to_dts(&self, indent_level: u32) -> String {
        let mut indents = String::new();
        for i in 0..indent_level {
            indents.push('\t')
        }
        let mut s = String::from(format!("{indents}{} = ", self.name));
        for i in 0..self.value.len() {
            let seg = &self.value[i];
            if i > 0 {
                s.push(',')
            }
            s.push('\"');
            s.push_str(seg);
            s.push('\"');
        }
        s.push(';');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_none() {
        let attr = Attribute::new("name", None);
        println!("{}", attr.to_dts(0));
    }

    #[test]
    fn test_attribute_u32() {
        let attr = Attribute::new("name", 42u32);
        println!("{}", attr.to_dts(0));
    }

    #[test]
    fn test_attribute_strs() {
        let string1 = String::from("&str abc");
        let string2 = String::from("def");
        let strs = vec![string1, string2];
        let attr = Attribute::new("name", strs);
        println!("{}", attr.to_dts(0));
    }

    #[test]
    fn test_attribute_str() {
        let s = String::from("&str abc");
        let attr = Attribute::new("name", s);
        println!("{}", attr.to_dts(0));
    }

    #[test]
    fn test_attribute_bytes() {
        let bytes = vec![0u8, 1u8, 2u8, 3u8];
        let attr = Attribute::new("name", bytes);
        println!("{}", attr.to_dts(0));
    }
}
