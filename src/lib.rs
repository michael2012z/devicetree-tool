// Copyright (c) 2023, Michael Zhao
// SPDX-License-Identifier: MIT

pub mod devicetree;
pub use devicetree::DeviceTree;
pub mod dtb;
pub use dtb::DtbHeader;
mod dtb_generator;
mod dtb_parser;
mod dts_generator;
mod dts_parser;
pub mod node;
pub use node::Node;
pub mod property;
pub use property::Property;
pub mod reservation;
pub use reservation::Reservation;
mod utils;
