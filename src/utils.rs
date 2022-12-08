// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

pub struct Utils {}

impl Utils {
    pub fn indent(level: u32) -> String {
        let mut s = String::new();
        for _ in 0..level {
            s.push('\t')
        }
        s
    }
}
