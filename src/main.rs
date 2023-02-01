// Copyright (c) 2022, Michael Zhao
// SPDX-License-Identifier: MIT

use argh::FromArgs;
use devicetree_tool::tree::Tree;

#[derive(FromArgs)]
/// Device tree tool
struct Args {
    /// input type
    #[argh(option)]
    in_type: String,

    /// input filename
    #[argh(option)]
    in_file: String,

    /// output type
    #[argh(option)]
    out_type: String,

    /// output filename
    #[argh(option)]
    out_file: String,
}

fn main() {
    let args: Args = argh::from_env();

    if &args.in_type != "dts" && &args.in_type != "dtb" {
        println!("Invalid input type");
    } else if &args.out_type != "dts" && &args.out_type != "dtb" {
        println!("Invalid output type");
    } else if &args.in_type == &args.out_type {
        println!("Input type and output type cannot be same");
    } else if &args.in_type == "dts" && &args.out_type == "dtb" {
        println!("Encode DTS ({}) to DTB ({})", args.in_file, args.out_file);

        let dts = std::fs::read_to_string(&args.in_file).expect("Unable to read input file");
        let tree = Tree::from_dts_bytes(dts.as_bytes());
        let dtb = tree.generate_dtb();
        std::fs::write(&args.out_file, dtb).expect("Unable to write output file");
    } else if &args.in_type == "dtb" && &args.out_type == "dts" {
        println!("Decode DTB ({}) to DTS ({})", args.in_file, args.out_file);

        let dtb = std::fs::read(&args.in_file).expect("Unable to read input file");
        let tree = Tree::from_dtb_bytes(&dtb);
        let dts = tree.generate_dts();
        std::fs::write(&args.out_file, dts).expect("Unable to write output file");
    } else {
        println!("Invalid input or output type");
    }
}
