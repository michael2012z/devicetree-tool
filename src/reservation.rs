// Copyright (c) 2023, Michael Zhao
// SPDX-License-Identifier: MIT

use crate::dts_generator::DtsGenerator;

/// A memory reservation block for reserving physical memory.
#[derive(Copy, Clone)]
pub struct Reservation {
    pub address: u64,
    pub length: u64,
}

/// Create a new memory reservation block with physical address and length
///
/// Example:
///
/// ```
/// use devicetree_tool::Reservation;
///
/// let resv = Reservation::new(0, 0x1000);
///
/// assert_eq!(format!("{}", resv), "/memreserve/ 0x0000000000000000 0x0000000000001000;");
/// ```
impl Reservation {
    pub fn new(address: u64, length: u64) -> Self {
        Reservation { address, length }
    }
}

impl std::fmt::Display for Reservation {
    /// Print a `Property` in the format of DTS
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
