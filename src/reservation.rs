// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dts_generator::DtsGenerator;

#[derive(Copy, Clone)]
pub struct Reservation {
    pub address: u64,
    pub length: u64,
}

impl Reservation {
    pub fn new(address: u64, length: u64) -> Self {
        Reservation { address, length }
    }
}

impl std::fmt::Display for Reservation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = DtsGenerator::generate_reservation(self, 0);
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reservation_print() {
        let reservation = Reservation::new(0x0, 0x100000);
        let printing = format!("{}", reservation);
        assert_eq!(
            &printing,
            "/memreserve/ 0x0000000000000000 0x0000000000100000;"
        );
    }
}
