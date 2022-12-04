// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use core::{fmt, fmt::Display};

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

impl Display for Attribute<u32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl Display for Attribute<f32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl Display for Attribute<&str> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl Display for Attribute<&Vec<&str>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for seg in self.value {
            if s.len() > 0 {
                s.push('~')
            }
            s.push_str(seg);
        }
        write!(f, "{}: {}", self.name, s)
    }
}

impl Display for Attribute<&[u8]> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for u in self.value {
            s.push(*u as char)
        }
        write!(f, "{}: {}", self.name, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_attribute_u32() {
        let attr = Attribute::new("name", 42u32);
        println!("{attr}");
    }

    #[test]
    fn test_attribute_f32() {
        let attr = Attribute::new("name", 12.3456f32);
        println!("{attr}");
    }

    #[test]
    fn test_attribute_strs() {
        let string1 = String::from("&str abc");
        let string2 = String::from("def");
        let strs = vec![string1.as_str(), string2.as_str()];
        let attr = Attribute::new("name", &strs);
        println!("{attr}");
    }

    #[test]
    fn test_attribute_str() {
        let string = String::from("&str abc");
        let s = string.as_str();
        let attr = Attribute::new("name", s);
        println!("{attr}");
    }

    #[test]
    fn test_attribute_bytes() {
        let data_string = String::from("bytes xyz");
        let bytes = data_string.as_bytes();
        let attr = Attribute::new("name", bytes);
        println!("{attr}");
    }
}
