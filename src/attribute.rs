// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

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

pub trait InternalAttribute {
    fn to_dts(&self) -> String;
}

impl<T> InternalAttribute for Attribute<Option<T>> {
    fn to_dts(&self) -> String {
        String::from(format!("{}: None", self.name))
    }
}

impl InternalAttribute for Attribute<u32> {
    fn to_dts(&self) -> String {
        String::from(format!("{}: {}", self.name, self.value))
    }
}

impl InternalAttribute for Attribute<f32> {
    fn to_dts(&self) -> String {
        String::from(format!("{}: {}", self.name, self.value))
    }
}

impl InternalAttribute for Attribute<&str> {
    fn to_dts(&self) -> String {
        String::from(format!("{}: {}", self.name, self.value))
    }
}

impl InternalAttribute for Attribute<&[u8]> {
    fn to_dts(&self) -> String {
        let mut s = String::from(format!("{}: ", self.name));
        for u in self.value {
            s.push(*u as char)
        }
        s
    }
}

impl InternalAttribute for Attribute<&Vec<&str>> {
    fn to_dts(&self) -> String {
        let mut s = String::from(format!("{}: ", self.name));
        for seg in self.value {
            if s.len() > 0 {
                s.push('~')
            }
            s.push_str(seg);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_none() {
        let attr: Attribute<Option<u32>> = Attribute::new("name", None);
        println!("{}", attr.to_dts());
    }

    #[test]
    fn test_attribute_u32() {
        let attr = Attribute::new("name", 42u32);
        println!("{}", attr.to_dts());
    }

    #[test]
    fn test_attribute_f32() {
        let attr = Attribute::new("name", 12.3456f32);
        println!("{}", attr.to_dts());
    }

    #[test]
    fn test_attribute_strs() {
        let string1 = String::from("&str abc");
        let string2 = String::from("def");
        let strs = vec![string1.as_str(), string2.as_str()];
        let attr = Attribute::new("name", &strs);
        println!("{}", attr.to_dts());
    }

    #[test]
    fn test_attribute_str() {
        let string = String::from("&str abc");
        let s = string.as_str();
        let attr = Attribute::new("name", s);
        println!("{}", attr.to_dts());
    }

    #[test]
    fn test_attribute_bytes() {
        let data_string = String::from("bytes xyz");
        let bytes = data_string.as_bytes();
        let attr = Attribute::new("name", bytes);
        println!("{}", attr.to_dts());
    }
}
