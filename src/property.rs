// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dts_generator::DtsGenerator;

/// A property that describes a characteristic of node.
///
/// # Examples
///
/// You can create an empty property with a name:
///
/// ```
/// use devicetree_tool::property::Property;
///
/// let prop = Property::new_empty("prop");
///
/// assert_eq!(prop.value, vec![]);
/// ```
///
/// Or create a property with value in the type of `u32`, `u64`, `str` or others.
///
/// ```
/// use devicetree_tool::property::Property;
///
/// let prop = Property::new_u32("prop", 42);
///
/// assert_eq!(format!("{}", prop), "prop = <0x0 0x0 0x0 0x2a>;\n");
/// ```
pub struct Property {
    pub name: String,
    pub value: Vec<u8>,
}

impl Property {
    /// Create a `Property` with a name, but without any value.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::property::Property;
    ///
    /// let prop = Property::new_empty("prop");
    ///
    /// assert_eq!(prop.value, vec![]);
    /// assert_eq!(format!("{}", prop), "prop;\n");
    /// ```
    pub fn new_empty(name: &str) -> Self {
        Property {
            name: String::from(name),
            value: vec![],
        }
    }

    /// Create a named `Property` with the value of type `u32`.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::property::Property;
    ///
    /// let prop = Property::new_u32("prop", 42);
    ///
    /// assert_eq!(prop.value, vec![0u8, 0u8, 0u8, 42u8]);
    /// assert_eq!(format!("{}", prop), "prop = <0x0 0x0 0x0 0x2a>;\n");
    /// ```
    pub fn new_u32(name: &str, value: u32) -> Self {
        Property {
            name: String::from(name),
            value: value.to_be_bytes().to_vec(),
        }
    }

    /// Create a named `Property` with the value of type `u64`.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::property::Property;
    ///
    /// let prop = Property::new_u64("prop", 42);
    ///
    /// assert_eq!(prop.value, vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 42u8]);
    /// assert_eq!(format!("{}", prop), "prop = <0x0 0x0 0x0 0x0 0x0 0x0 0x0 0x2a>;\n");
    /// ```
    pub fn new_u64(name: &str, value: u64) -> Self {
        Property {
            name: String::from(name),
            value: value.to_be_bytes().to_vec(),
        }
    }

    /// Create a named `Property` with the value of string.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::property::Property;
    ///
    /// let prop = Property::new_str("prop", "hello");
    ///
    /// assert_eq!(prop.value, vec!['h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0u8]);
    /// assert_eq!(format!("{}", prop), "prop = <0x68 0x65 0x6c 0x6c 0x6f 0x0>;\n");
    /// ```
    pub fn new_str(name: &str, value: &str) -> Self {
        let mut bytes: Vec<u8> = value.as_bytes().to_vec();
        bytes.push(0);
        Property {
            name: String::from(name),
            value: bytes,
        }
    }

    /// Create a named `Property` with the value of a string list.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::property::Property;
    ///
    /// let prop = Property::new_strs("prop", vec!["hello", "abc"]);
    ///
    /// assert_eq!(prop.value, vec!['h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0u8, 'a' as u8, 'b' as u8, 'c' as u8, 0u8]);
    /// assert_eq!(format!("{}", prop), "prop = <0x68 0x65 0x6c 0x6c 0x6f 0x0 0x61 0x62 0x63 0x0>;\n");
    /// ```
    pub fn new_strs(name: &str, value: Vec<&str>) -> Self {
        let mut bytes: Vec<u8> = vec![];
        for v in value {
            let mut v_bytes = v.as_bytes().to_vec();
            bytes.append(&mut v_bytes);
            bytes.push(0)
        }
        Property {
            name: String::from(name),
            value: bytes,
        }
    }

    /// Create a named `Property` with the value of a `u8` array.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::property::Property;
    ///
    /// let prop = Property::new_u8s("prop", vec![1u8, 2u8, 3u8, 4u8]);
    ///
    /// assert_eq!(prop.value, vec![1u8, 2u8, 3u8, 4u8]);
    /// assert_eq!(format!("{}", prop), "prop = <0x1 0x2 0x3 0x4>;\n");
    /// ```
    pub fn new_u8s(name: &str, value: Vec<u8>) -> Self {
        Property {
            name: String::from(name),
            value,
        }
    }

    /// Create a named `Property` with the value of a `u32` array.
    ///
    /// # Example
    ///
    /// ```
    /// use devicetree_tool::property::Property;
    ///
    /// let prop = Property::new_u32s("prop", vec![1u32, 2u32]);
    ///
    /// assert_eq!(prop.value, vec![0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 2u8]);
    /// assert_eq!(format!("{}", prop), "prop = <0x0 0x0 0x0 0x1 0x0 0x0 0x0 0x2>;\n");
    /// ```
    pub fn new_u32s(name: &str, value: Vec<u32>) -> Self {
        let mut bytes: Vec<u8> = vec![];
        for v in value {
            let mut v_bytes = v.to_be_bytes().to_vec();
            bytes.append(&mut v_bytes)
        }
        Property {
            name: String::from(name),
            value: bytes,
        }
    }
}

impl std::fmt::Display for Property {
    /// Print a `Property` in the format of DTS
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = DtsGenerator::generate_property(self, 0);
        writeln!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_property_none() {
        let prop = Property::new_empty("name");
        assert_eq!(prop.value, vec![]);
    }

    #[test]
    fn test_property_u32() {
        let prop = Property::new_u32("name", 42);
        assert_eq!(prop.value, vec![0u8, 0u8, 0u8, 42u8]);
    }

    #[test]
    fn test_property_u64() {
        let prop = Property::new_u64("name", 42);
        assert_eq!(prop.value, vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 42u8]);
    }

    #[test]
    fn test_property_str() {
        let prop = Property::new_str("name", "hello abc");
        assert_eq!(
            prop.value,
            vec![
                'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, ' ' as u8, 'a' as u8,
                'b' as u8, 'c' as u8, 0
            ]
        );
    }

    #[test]
    fn test_property_strs() {
        let strs = vec!["hello", "abc"];
        let prop = Property::new_strs("name", strs);
        assert_eq!(
            prop.value,
            vec![
                'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 'a' as u8, 'b' as u8,
                'c' as u8, 0
            ]
        );
    }

    #[test]
    fn test_property_u8s() {
        let bytes = vec![0u8, 1u8, 2u8, 3u8];
        let prop = Property::new_u8s("name", bytes);
        assert_eq!(prop.value, vec![0u8, 1u8, 2u8, 3u8]);
    }

    #[test]
    fn test_property_u32s() {
        let bytes = vec![0u32, 1u32, 2u32, 3u32];
        let prop = Property::new_u32s("name", bytes);
        assert_eq!(
            prop.value,
            vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 3u8]
        );
    }

    #[test]
    fn test_property_print() {
        let prop = Property::new_u32("name", 42);
        let printing = format!("{}", prop);
        assert_eq!(&printing, "name = <0x0 0x0 0x0 0x2a>;\n");
    }
}
