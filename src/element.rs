// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

//
// Internal generic-purposed element for all types data of the device tree:
//   * Tree
//   * Node
//   * Property
//
pub trait Element {
    fn to_dts(&self, indent_level: u32) -> String;
    fn to_dtb(&self) -> Vec<u8> {
        unimplemented!()
    }
}
