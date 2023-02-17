# devicetree-tool

A device tree building and parsing tool written in Rust

## General

`devicetree-tool` is both:
- A Rust crate that can be used for manipulating device trees
- A command line tool based on the crate that can be used to encode and decode device tree files

The center of `devicetree-tool` is the meta data consisting the device tree:
- `DeviceTree` - The top level structure of the device tree meta data
- `Node` - A node in the device tree, representing a device
- `Property` - A property item of the device node
- `Reservation` - Physical memeory reservation block of the device tree

The device tree meta data can be created from the source code, or be built from the content of DTS or DTB files. The meta data can also be managed in source code, or be encoded to DTS or DTB format.

## `devicetree-tool` Crate

### Examples

Create a device tree from scratch:

``` rust
use devicetree_tool::Node;
use devicetree_tool::DeviceTree;

fn main() {
    // Create the root node
    let mut node = Node::new("");

    // Add a property
    node.add_property(Property::new_u32("prop", 42));

    // Add a sub node
    node.add_sub_node(Node::new("sub_node"));

    // Create the device tree from the root node
    let tree = DeviceTree::new(vec![], node);

    assert_eq!(
        format!("{}", tree),
        "/dts-v1/;\n\n/ {\n\tprop = <0x0 0x0 0x0 0x2a>;\n\n\tsub_node {\n\t};\n};\n\n"
    );
}
```

## `devicetree-tool` Crate

### Examples

Encode a DTS file to DTB:

``` bash
# Build devicetree-tool
cargo build --release

# Create a simple DTS file
cat << EOF > ./temp.dts
/dts-v1/;

/ {
	cpus {
		#address-cells = <0x02>;
		#size-cells = <0x00>;

		cpu@0 {
			device_type = "cpu";
			compatible = "arm,arm-v8";
			enable-method = "psci";
			reg = <0x00 0x00>;
		};
	};
};
EOF

# Decode the DTS file into DTB
./target/release/devicetree-tool --in-type dts --in-file ./temp.dts --out-type dtb --out-file ./temp.dtb
```