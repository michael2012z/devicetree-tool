// Copyright (c) 2023, Michael Zhao
// SPDX-License-Identifier: MIT

mod devicetree;
pub use devicetree::DeviceTree;
mod dtb;
mod dtb_generator;
mod dtb_parser;
mod dts_generator;
mod dts_parser;
mod node;
pub use node::Node;
mod property;
pub use property::Property;
mod reservation;
pub use reservation::Reservation;
mod utils;
